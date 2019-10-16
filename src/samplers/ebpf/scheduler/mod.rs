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

use std::sync::Arc;

pub struct Scheduler {
    bpf: BPF,
    common: Common,
}

impl Sampler for Scheduler {
    fn new(config: Arc<Config>, metrics: Metrics<AtomicU32>) -> Result<Option<Box<Self>>, Error> {
        debug!("initializing");
        // load the code and compile
        let code = include_str!("bpf.c");
        let mut bpf = BPF::new(code)?;

        // load + attach kprobes!
        let trace_run = bpf.load_kprobe("trace_run")?;
        let trace_ttwu_do_wakeup = bpf.load_kprobe("trace_ttwu_do_wakeup")?;
        let trace_wake_up_new_task = bpf.load_kprobe("trace_wake_up_new_task")?;

        bpf.attach_kprobe("finish_task_switch", trace_run)?;
        bpf.attach_kprobe("wake_up_new_task", trace_wake_up_new_task)?;
        bpf.attach_kprobe("ttwu_do_wakeup", trace_ttwu_do_wakeup)?;

        Ok(Some(Box::new(Self {
            bpf,
            common: Common::new(config, metrics),
        }) as Box<dyn Sampler>))
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn name(&self) -> String {
        "ebpf::scheduler".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        // gather current state
        trace!("sampling {}", self.name());
        self.register();
        let time = time::precise_time_ns();
        for statistic in &[Statistic::RunqueueLatency] {
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
        debug!("register {}", self.name());
        if !self.common.initialized() {
            self.common
                .register_distribution(&Statistic::RunqueueLatency, SECOND, 2, PERCENTILES);
            self.common.set_initialized(true);
        }
    }

    fn deregister(&mut self) {
        debug!("deregister {}", self.name());
        self.common.delete_channel(&Statistic::RunqueueLatency);
        self.common.set_initialized(false);
    }
}
