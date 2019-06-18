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

pub struct Ext4 {
    bpf: BPF,
    initialized: bool,
}

impl Ext4 {
    pub fn new() -> Result<Self, Error> {
        debug!("initializing");
        // load the code and compile
        let code = include_str!("ext4dist.c").to_string();
        let addr = "0x".to_string() + &super::symbol_lookup("ext4_file_operations").unwrap();
        let code = code.replace("EXT4_FILE_OPERATIONS", &addr);
        let mut bpf = BPF::new(&code)?;

        // load + attach kprobes!

        let generic_file_read_iter_entry = bpf.load_kprobe("trace_read_entry")?;
        let ext4_file_write_iter_entry = bpf.load_kprobe("trace_entry")?;
        let ext4_file_open_entry = bpf.load_kprobe("trace_entry")?;
        let ext4_sync_file_entry = bpf.load_kprobe("trace_entry")?;
        let generic_file_read_iter_return = bpf.load_kprobe("trace_read_return")?;
        let ext4_file_write_iter_return = bpf.load_kprobe("trace_write_return")?;
        let ext4_file_open_return = bpf.load_kprobe("trace_open_return")?;
        let ext4_sync_file_return = bpf.load_kprobe("trace_fsync_return")?;

        bpf.attach_kprobe("generic_file_read_iter", generic_file_read_iter_entry)?;
        bpf.attach_kprobe("ext4_file_write_iter", ext4_file_write_iter_entry)?;
        bpf.attach_kprobe("ext4_file_open", ext4_file_open_entry)?;
        bpf.attach_kprobe("ext4_sync_file", ext4_sync_file_entry)?;
        bpf.attach_kretprobe("generic_file_read_iter", generic_file_read_iter_return)?;
        bpf.attach_kretprobe("ext4_file_write_iter", ext4_file_write_iter_return)?;
        bpf.attach_kretprobe("ext4_file_open", ext4_file_open_return)?;
        bpf.attach_kretprobe("ext4_sync_file", ext4_sync_file_return)?;

        Ok(Self {
            bpf,
            initialized: false,
        })
    }
}

impl Sampler for Ext4 {
    fn name(&self) -> String {
        "ebpf::ext4".to_string()
    }

    fn sample(&mut self, recorder: &Recorder<u32>, config: &Config) -> Result<(), SamplerError> {
        // gather current state
        trace!("sampling ebpf::ext4");
        let time = time::precise_time_ns();

        if !self.initialized {
            self.register(recorder, config);
        }
        for stat in &["read", "write", "open", "fsync"] {
            trace!("sampling ebpf::ext4::{}", stat);
            let mut table = self.bpf.table(stat);
            for (&latency, &count) in &map_from_table(&mut table) {
                record_distribution(recorder, format!("ext4/{}", stat), time, latency, count);
            }
        }
        Ok(())
    }

    fn register(&mut self, recorder: &Recorder<u32>, config: &Config) {
        debug!("register {}", self.name());
        if !self.initialized {
            for label in ["ext4/read", "ext4/write", "ext4/open", "ext4/fsync"].iter() {
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
            for label in ["ext4/read", "ext4/write", "ext4/open", "ext4/fsync"].iter() {
                recorder.delete_channel(label.to_string());
            }
            self.initialized = false;
        }
    }
}

fn map_from_table(table: &mut Table) -> HashMap<u64, u32> {
    let mut current = HashMap::new();

    trace!("transferring data to userspace");
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
