// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod statistics;

use self::statistics::{Direction, Statistic};
use crate::common::{BILLION, MICROSECOND, MILLION, PERCENTILES};
use crate::config::Config;
use crate::samplers::{Common, Sampler};

use bcc;
use bcc::core::BPF;
use failure::*;
use logger::*;
use metrics::*;
use time;

use std::collections::HashMap;

pub struct Block<'a> {
    bpf: BPF,
    common: Common<'a>,
}

impl<'a> Block<'a> {
    fn report_statistic(&mut self, statistic: &Statistic) {
        match statistic {
            Statistic::Size(_) => {
                self.report_size(&statistic.table_name(), &statistic.to_string());
            }
            _ => {
                self.report_latency(&statistic.table_name(), &statistic.to_string());
            }
        }
    }

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
        self.register();
        for (&value, &count) in &current {
            self.common.record_distribution(&label, time, value, count);
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
        self.register();
        for (&value, &count) in &current {
            self.common.record_distribution(&label, time, value, count);
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
            bpf,
            common: Common::new(config, recorder),
        })))
    }

    fn name(&self) -> String {
        "ebpf::block".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        // gather current state
        trace!("sampling {}", self.name());
        for statistic in &[
            Statistic::Size(Direction::Read),
            Statistic::Size(Direction::Write),
            Statistic::Latency(Direction::Read),
            Statistic::Latency(Direction::Write),
            Statistic::DeviceLatency(Direction::Read),
            Statistic::DeviceLatency(Direction::Write),
            Statistic::QueueLatency(Direction::Read),
            Statistic::QueueLatency(Direction::Write),
        ] {
            self.report_statistic(statistic);
        }
        Ok(())
    }

    fn register(&mut self) {
        if !self.common.initialized() {
            debug!("register {}", self.name());
            for statistic in &[
                Statistic::Size(Direction::Read),
                Statistic::Size(Direction::Write),
            ] {
                self.common
                    .register_distribution(statistic, MILLION, 2, PERCENTILES);
            }
            for size in &["block/size/read", "block/size/write"] {
                self.common
                    .register_distribution(size, MILLION, 2, PERCENTILES);
            }
            for statistic in &[
                Statistic::Latency(Direction::Read),
                Statistic::Latency(Direction::Write),
                Statistic::DeviceLatency(Direction::Read),
                Statistic::DeviceLatency(Direction::Write),
                Statistic::QueueLatency(Direction::Read),
                Statistic::QueueLatency(Direction::Write),
            ] {
                self.common
                    .register_distribution(statistic, BILLION, 2, PERCENTILES);
            }
            self.common.set_initialized(true);
        }
    }

    fn deregister(&mut self) {
        if self.common.initialized() {
            trace!("deregister {}", self.name());
            for statistic in &[
                Statistic::Size(Direction::Read),
                Statistic::Size(Direction::Write),
                Statistic::Latency(Direction::Read),
                Statistic::Latency(Direction::Write),
                Statistic::DeviceLatency(Direction::Read),
                Statistic::DeviceLatency(Direction::Write),
                Statistic::QueueLatency(Direction::Read),
                Statistic::QueueLatency(Direction::Write),
            ] {
                self.common.delete_channel(statistic);
            }
            self.common.set_initialized(false);
        }
    }
}
