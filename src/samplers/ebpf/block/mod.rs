// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod statistics;

use self::statistics::{Direction, Statistic};
use super::map_from_table;
use crate::common::{BILLION, MICROSECOND, MILLION, PERCENTILES, UNITY};
use crate::config::*;
use crate::samplers::{Common, Sampler};

use bcc;
use bcc::core::BPF;
use failure::*;
use logger::*;
use metrics::*;
use time;

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
        let mut table = self.bpf.table(table);

        self.register();

        for (&value, &count) in &map_from_table(&mut table, MICROSECOND) {
            self.common.record_distribution(&label, time, value, count);
        }
    }

    fn report_size(&mut self, table: &str, label: &str) {
        let time = time::precise_time_ns();
        let mut table = self.bpf.table(table);

        self.register();

        for (&value, &count) in &map_from_table(&mut table, UNITY) {
            self.common.record_distribution(&label, time, value, count);
        }
    }
}

impl<'a> Sampler<'a> for Block<'a> {
    fn new(
        config: &'a Config,
        metrics: &'a Metrics<AtomicU32>,
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
            common: Common::new(config, metrics),
        })))
    }

    fn common(&self) -> &Common<'a> {
        &self.common
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

    fn interval(&self) -> usize {
        self.common()
            .config()
            .ebpf()
            .interval()
            .unwrap_or_else(|| self.common().config().interval())
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
