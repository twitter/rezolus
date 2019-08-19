// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::{BILLION, MICROSECOND, MILLION, PERCENTILES};
use crate::config::Config;
use crate::samplers::Sampler;
use crate::stats::{record_distribution, register_distribution};

use bcc;
use bcc::core::BPF;
use failure::*;
use logger::*;
use metrics::*;
use time;

use std::collections::HashMap;

pub struct Block<'a> {
    config: &'a Config,
    bpf: BPF,
    initialized: bool,
    recorder: &'a Recorder<AtomicU32>,
}

impl<'a> Block<'a> {
    fn report_latency(&mut self, table: &str, label: &str) {
        let time = time::precise_time_ns();
        let mut current = HashMap::new();
        let mut data = self.bpf.table(table);

        for mut entry in data.iter() {
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
            let _ = data.set(&mut entry.key, &mut [0_u8; 8]);
        }
        if !self.initialized {
            self.register();
        } else {
            for (&value, &count) in &current {
                record_distribution(self.recorder, label, time, value, count);
            }
        }
    }

    fn report_size(&mut self, table: &str, label: &str) {
        let time = time::precise_time_ns();
        let mut current = HashMap::new();
        let mut data = self.bpf.table(table);

        for mut entry in data.iter() {
            let mut key = [0; 4];
            key.copy_from_slice(&entry.key);
            let key = u32::from_ne_bytes(key);

            let mut value = [0; 8];
            value.copy_from_slice(&entry.value);
            let value = u64::from_ne_bytes(value);

            if let Some(key) = super::key_to_value(key as u64) {
                current.insert(key, value as u32);
            }

            // clear the source counter
            let _ = data.set(&mut entry.key, &mut [0_u8; 8]);
        }
        if !self.initialized {
            self.register();
        } else {
            for (&value, &count) in &current {
                record_distribution(self.recorder, label, time, value, count);
            }
        }
    }
}

impl<'a> Sampler<'a> for Block<'a> {
    fn new(
        config: &'a Config,
        recorder: &'a Recorder<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
        debug!("initializing");
        // load the code and compile
        let code = include_str!("bpf.c");
        let mut bpf = BPF::new(code)?;
        // load + attach kprobes!
        let trace_pid_start = bpf.load_kprobe("trace_pid_start")?;
        let trace_req_start = bpf.load_kprobe("trace_req_start")?;
        let trace_mq_req_start = bpf.load_kprobe("trace_req_start")?;
        let do_count = bpf.load_kprobe("do_count")?;

        bpf.attach_kprobe("blk_account_io_start", trace_pid_start)?;
        bpf.attach_kprobe("blk_start_request", trace_req_start)?;
        bpf.attach_kprobe("blk_mq_start_request", trace_mq_req_start)?;
        bpf.attach_kprobe("blk_account_io_completion", do_count)?;

        Ok(Some(Box::new(Self {
            config,
            bpf,
            initialized: false,
            recorder,
        })))
    }

    fn name(&self) -> String {
        "ebpf::block".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        // gather current state
        trace!("sampling {}", self.name());
        self.report_size("read_size", "block/size/read");
        self.report_size("write_size", "block/size/write");
        self.report_latency("read_latency", "block/latency/read");
        self.report_latency("read_request_latency", "block/device_latency/read");
        self.report_latency("read_queue_latency", "block/queue_latency/read");
        self.report_latency("write_latency", "block/latency/write");
        self.report_latency("write_request_latency", "block/device_latency/write");
        self.report_latency("write_queue_latency", "block/queue_latency/write");
        Ok(())
    }

    fn register(&mut self) {
        debug!("register {}", self.name());
        if !self.initialized {
            for size in &["block/size/read", "block/size/write"] {
                register_distribution(
                    self.recorder,
                    size,
                    MILLION,
                    2,
                    self.config.general().window(),
                    PERCENTILES,
                );
            }
            for latency in &[
                "block/latency/read",
                "block/device_latency/read",
                "block/queue_latency/read",
                "block/latency/write",
                "block/device_latency/write",
                "block/queue_latency/write",
            ] {
                register_distribution(
                    self.recorder,
                    latency,
                    BILLION,
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
            self.recorder.delete_channel("block/size/read".to_string());
            self.recorder.delete_channel("block/size/write".to_string());
            self.recorder
                .delete_channel("block/latency/read".to_string());
            self.recorder
                .delete_channel("block/device_latency/read".to_string());
            self.recorder
                .delete_channel("block/queue_latency/read".to_string());
            self.recorder
                .delete_channel("block/latency/write".to_string());
            self.recorder
                .delete_channel("block/device_latency/write".to_string());
            self.recorder
                .delete_channel("block/queue_latency/write".to_string());
            self.initialized = false;
        }
    }
}
