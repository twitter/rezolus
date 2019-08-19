// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::{MICROSECOND, PERCENTILES, SECOND};
use crate::config::Config;
use crate::samplers::Sampler;
use crate::stats::{record_distribution, register_distribution};

use bcc;
use bcc::core::BPF;
use bcc::table::Table;
use failure::*;
use logger::*;
use metrics::*;
use time;

use std::collections::HashMap;

pub struct Xfs<'a> {
    bpf: BPF,
    config: &'a Config,
    initialized: bool,
    recorder: &'a Recorder<AtomicU32>,
}

impl<'a> Sampler<'a> for Xfs<'a> {
    fn new(
        config: &'a Config,
        recorder: &'a Recorder<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
        debug!("initializing");
        // load the code and compile
        let code = include_str!("bpf.c").to_string();
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

        Ok(Some(Box::new(Self {
            bpf,
            config,
            initialized: false,
            recorder,
        })))
    }

    fn name(&self) -> String {
        "ebpf::xfs".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        // gather current state
        trace!("sample {}", self.name());
        let time = time::precise_time_ns();

        if !self.initialized {
            self.register();
        }
        for stat in &["read", "write", "open", "fsync"] {
            let mut table = self.bpf.table(stat);
            for (&latency, &count) in &map_from_table(&mut table) {
                record_distribution(self.recorder, format!("xfs/{}", stat), time, latency, count);
            }
        }
        Ok(())
    }

    fn register(&mut self) {
        debug!("register {}", self.name());
        if !self.initialized {
            for label in ["xfs/read", "xfs/write", "xfs/open", "xfs/fsync"].iter() {
                register_distribution(
                    self.recorder,
                    label,
                    SECOND,
                    2,
                    self.config.general().window(),
                    PERCENTILES,
                );
            }
            self.initialized = true;
        }
    }

    fn deregister(&mut self) {
        debug!("deregister {}", self.name());
        if self.initialized {
            for label in ["xfs/read", "xfs/write", "xfs/open", "xfs/fsync"].iter() {
                self.recorder.delete_channel(label.to_string());
            }
            self.initialized = false;
        }
    }
}

fn map_from_table(table: &mut Table) -> HashMap<u64, u32> {
    let mut current = HashMap::new();
    for mut entry in table.iter() {
        let mut key = [0; 4];
        key.copy_from_slice(&entry.key);
        let key = u32::from_ne_bytes(key);

        let mut value = [0; 8];
        value.copy_from_slice(&entry.value);
        let value = u64::from_ne_bytes(value);

        if let Some(key) = super::key_to_value(key as u64) {
            current.insert(key * MICROSECOND, value as u32);
        }

        // clear the source counter
        let _ = table.set(&mut entry.key, &mut [0_u8; 8]);
    }
    current
}
