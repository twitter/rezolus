// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::{MICROSECOND, PERCENTILES, SECOND};
use crate::config::Config;
use crate::samplers::{Common, Sampler};

use bcc;
use bcc::core::BPF;
use bcc::table::Table;
use failure::*;
use logger::*;
use metrics::*;
use time;

use std::collections::HashMap;

pub struct Ext4<'a> {
    bpf: BPF,
    common: Common<'a>,
    initialized: bool,
}

impl<'a> Sampler<'a> for Ext4<'a> {
    fn new(
        config: &'a Config,
        recorder: &'a Recorder<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
        debug!("initializing");
        // load the code and compile
        let code = include_str!("bpf.c").to_string();
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

        Ok(Some(Box::new(Self {
            bpf,
            common: Common::new(config, recorder),
            initialized: false,
        })))
    }

    fn name(&self) -> String {
        "ebpf::ext4".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        // gather current state
        trace!("sampling ebpf::ext4");
        let time = time::precise_time_ns();

        if !self.initialized {
            self.register();
        }
        for stat in &["read", "write", "open", "fsync"] {
            trace!("sampling ebpf::ext4::{}", stat);
            let mut table = self.bpf.table(stat);
            for (&latency, &count) in &map_from_table(&mut table) {
                self.common
                    .record_distribution(&format!("ext4/{}", stat), time, latency, count);
            }
        }
        Ok(())
    }

    fn register(&mut self) {
        debug!("register {}", self.name());
        if !self.initialized {
            for label in ["ext4/read", "ext4/write", "ext4/open", "ext4/fsync"].iter() {
                self.common
                    .register_distribution(label, SECOND, 2, PERCENTILES);
            }
            self.initialized = true;
        }
    }

    fn deregister(&mut self) {
        debug!("deregister {}", self.name());
        if self.initialized {
            for label in ["ext4/read", "ext4/write", "ext4/open", "ext4/fsync"].iter() {
                self.common.delete_channel(label);
            }
            self.initialized = false;
        }
    }
}

fn map_from_table(table: &mut Table) -> HashMap<u64, u32> {
    let mut current = HashMap::new();

    trace!("transferring data to userspace");
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
