// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::{MICROSECOND, PERCENTILES, SECOND};
use crate::config::Config;
use crate::samplers::{Common, Sampler};

use bcc;
use bcc::core::BPF;
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
        recorder: &'a Recorder<AtomicU32>,
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
            common: Common::new(config, recorder),
        })))
    }

    fn name(&self) -> String {
        "ebpf::scheduler".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        // gather current state
        trace!("sampling {}", self.name());
        let time = time::precise_time_ns();
        let mut current = HashMap::new();

        trace!("accessing data in kernelspace");
        let mut data = self.bpf.table("dist");

        trace!("copying data to userspace");
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
        trace!("data copied to userspace");
        self.register();
        for (&latency, &count) in &current {
            self.common
                .record_distribution(&"scheduler/runqueue_latency_ns", time, latency, count);
        }
        Ok(())
    }

    fn register(&mut self) {
        debug!("register {}", self.name());
        if !self.common.initialized() {
            self.common.register_distribution(
                &"scheduler/runqueue_latency_ns",
                SECOND,
                2,
                PERCENTILES,
            );
            self.common.set_initialized(true);
        }
    }

    fn deregister(&mut self) {
        debug!("deregister {}", self.name());
        if self.common.initialized() {
            self.common.delete_channel(&"scheduler/runqueue_latency_ns");
            self.common.set_initialized(false);
        }
    }
}
