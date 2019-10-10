// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod statistics;

use self::statistics::Statistic;
use super::map_from_table;
use crate::common::{MICROSECOND, PERCENTILES, SECOND};
use crate::config::*;
use crate::samplers::{Common, Sampler};

use bcc;
use bcc::core::BPF;
use failure::*;
use logger::*;
use metrics::*;
use time;

pub struct Tcp<'a> {
    bpf: BPF,
    common: Common<'a>,
}

impl<'a> Sampler<'a> for Tcp<'a> {
    fn new(
        config: &'a Config,
        metrics: &'a Metrics<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
        debug!("initializing");
        // load the code and compile
        let code = include_str!("bpf.c").to_string();
        let mut bpf = BPF::new(&code)?;

        // load + attach kprobes!
        let tcp_v4_connect = bpf.load_kprobe("trace_connect")?;
        let tcp_v6_connect = bpf.load_kprobe("trace_connect")?;
        let tcp_rcv_state_process = bpf.load_kprobe("trace_tcp_rcv_state_process")?;

        bpf.attach_kprobe("tcp_v4_connect", tcp_v4_connect)?;
        bpf.attach_kprobe("tcp_v6_connect", tcp_v6_connect)?;
        bpf.attach_kprobe("tcp_rcv_state_process", tcp_rcv_state_process)?;

        Ok(Some(Box::new(Self {
            bpf,
            common: Common::new(config, metrics),
        })))
    }

    fn common(&self) -> &Common<'a> {
        &self.common
    }

    fn name(&self) -> String {
        "ebpf::tcp".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        // gather current state
        trace!("sampling {}", self.name());
        let time = time::precise_time_ns();
        self.register();
        for statistic in &[Statistic::ConnectLatency] {
            trace!("sampling {}", statistic);
            let mut table = self.bpf.table(&statistic.table_name());
            for (&value, &count) in &map_from_table(&mut table, MICROSECOND) {
                self.common
                    .record_distribution(statistic, time, value, count);
            }
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
            trace!("register {}", self.name());
            for statistic in &[Statistic::ConnectLatency] {
                self.common
                    .register_distribution(statistic, SECOND, 2, PERCENTILES);
            }
            self.common.set_initialized(true);
        }
    }

    fn deregister(&mut self) {
        if self.common.initialized() {
            trace!("deregister {}", self.name());
            for statistic in &[Statistic::ConnectLatency] {
                self.common.delete_channel(statistic);
            }
            self.common.set_initialized(false);
        }
    }
}
