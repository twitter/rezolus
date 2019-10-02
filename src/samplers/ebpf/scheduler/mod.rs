// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod statistics;

use self::statistics::Statistic;
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

pub struct Scheduler<'a> {
    bpf: BPF,
    common: Common<'a>,
}

impl<'a> Sampler<'a> for Scheduler<'a> {
    fn new(
        config: &'a Config,
        metrics: &'a Metrics<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
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
        })))
    }

    fn common(&self) -> &Common<'a> {
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
            for (&value, &count) in &map_from_table(&mut table) {
                self.common
                    .record_distribution(statistic, time, value, count);
            }
        }
        Ok(())
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
        if self.common.initialized() {
            self.common.delete_channel(&Statistic::RunqueueLatency);
            self.common.set_initialized(false);
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
