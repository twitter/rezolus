// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::{MICROSECOND, PERCENTILES, SECOND};
use crate::config::Config;
use crate::sampler::{Sampler, SamplerError};
use crate::stats::{record_distribution, register_distribution};

use bcc;
use bcc::core::BPF;
use bcc::table::Table;
use failure::*;
use logger::*;
use metrics::*;
use time;

use std::collections::HashMap;
use std::mem;

pub struct Xfs {
    bpf: BPF,
    initialized: bool,
}

impl Xfs {
    pub fn new() -> Result<Self, Error> {
        debug!("initializing");
        // load the code and compile
        let code = include_str!("xfsdist.c").to_string();
        let mut bpf = BPF::new(&code)?;

        // load + attach kprobes!
        let read_entry = bpf.load_kprobe("trace_entry")?;
        let write_entry = bpf.load_kprobe("trace_entry")?;
        let open_entry = bpf.load_kprobe("trace_entry")?;
        let fsync_entry = bpf.load_kprobe("trace_entry")?;
        let read_return = bpf.load_kprobe("trace_read_return")?;
        let write_return = bpf.load_kprobe("trace_write_return")?;
        let open_return = bpf.load_kprobe("trace_open_return")?;
        let fsync_return = bpf.load_kprobe("trace_fsync_return")?;

        bpf.attach_kprobe("xfs_file_read_iter", read_entry)?;
        bpf.attach_kprobe("xfs_file_write_iter", write_entry)?;
        bpf.attach_kprobe("xfs_file_open", open_entry)?;
        bpf.attach_kprobe("xfs_file_fsync", fsync_entry)?;
        bpf.attach_kretprobe("xfs_file_read_iter", read_return)?;
        bpf.attach_kretprobe("xfs_file_write_iter", write_return)?;
        bpf.attach_kretprobe("xfs_file_open", open_return)?;
        bpf.attach_kretprobe("xfs_file_fsync", fsync_return)?;

        Ok(Self {
            bpf,
            initialized: false,
        })
    }
}

impl Sampler for Xfs {
    fn name(&self) -> String {
        "ebpf::xfs".to_string()
    }

    fn sample(&mut self, recorder: &Recorder<u32>, config: &Config) -> Result<(), SamplerError> {
        // gather current state
        trace!("sample {}", self.name());
        let time = time::precise_time_ns();

        if !self.initialized {
            self.register(recorder, config);
        }
        for stat in &["read", "write", "open", "fsync"] {
            let mut table = self.bpf.table(stat);
            for (&latency, &count) in &map_from_table(&mut table) {
                record_distribution(recorder, format!("xfs/{}", stat), time, latency, count);
            }
        }
        Ok(())
    }

    fn register(&mut self, recorder: &Recorder<u32>, config: &Config) {
        debug!("register {}", self.name());
        if !self.initialized {
            for label in ["xfs/read", "xfs/write", "xfs/open", "xfs/fsync"].iter() {
                register_distribution(
                    recorder,
                    label,
                    SECOND,
                    2,
                    config.general().interval(),
                    PERCENTILES,
                );
            }
            self.initialized = true;
        }
    }

    fn deregister(&mut self, recorder: &Recorder<u32>, _config: &Config) {
        debug!("deregister {}", self.name());
        if self.initialized {
            for label in ["xfs/read", "xfs/write", "xfs/open", "xfs/fsync"].iter() {
                recorder.delete_channel(label.to_string());
            }
            self.initialized = false;
        }
    }
}

fn map_from_table(table: &mut Table) -> HashMap<u64, u32> {
    let mut current = HashMap::new();
    for entry in table.iter() {
        let mut key = entry.key;
        let value = entry.value;

        // key is a u64 index into a histogram
        let mut k = [0_u8; 4];
        for (index, byte) in k.iter_mut().enumerate() {
            *byte = *key.get(index).unwrap_or(&0);
        }
        let k: u32 = unsafe { mem::transmute(k) };

        // convert the key to a block size in kbytes
        if let Some(key) = super::key_to_value(k as u64) {
            // value is a u32 count of times that block size was seen
            let mut v = [0_u8; 8];
            for (index, byte) in v.iter_mut().enumerate() {
                *byte = *value.get(index).unwrap_or(&0);
            }

            let count: u64 = unsafe { mem::transmute(v) };

            // store the size-count pair into the hashmap
            current.insert(key * MICROSECOND, count as u32);
        }

        // clear the source counter
        let _ = table.set(&mut key, &mut [0_u8; 8]);
    }
    current
}
