// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::{MICROSECOND, PERCENTILES, SECOND};
use crate::config::Config;
use crate::sampler::{Sampler, SamplerError};
use crate::stats::{record_distribution, register_distribution};

use bcc;
use bcc::core::BPF;
use failure::*;
use logger::*;
use metrics::*;
use time;

use std::collections::HashMap;
use std::mem;

pub struct Scheduler {
    bpf: BPF,
    initialized: bool,
}

impl Scheduler {
    pub fn new() -> Result<Self, Error> {
        debug!("initializing");
        // load the code and compile
        let code = include_str!("runqueue_latency.c");
        let mut bpf = BPF::new(code)?;

        // load + attach kprobes!
        let trace_run = bpf.load_kprobe("trace_run")?;
        let trace_ttwu_do_wakeup = bpf.load_kprobe("trace_ttwu_do_wakeup")?;
        let trace_wake_up_new_task = bpf.load_kprobe("trace_wake_up_new_task")?;

        bpf.attach_kprobe("finish_task_switch", trace_run)?;
        bpf.attach_kprobe("wake_up_new_task", trace_wake_up_new_task)?;
        bpf.attach_kprobe("ttwu_do_wakeup", trace_ttwu_do_wakeup)?;

        Ok(Self {
            bpf,
            initialized: false,
        })
    }
}

impl Sampler for Scheduler {
    fn name(&self) -> String {
        "ebpf::scheduler".to_string()
    }

    fn sample(&mut self, recorder: &Recorder<u32>, config: &Config) -> Result<(), SamplerError> {
        // gather current state
        trace!("sampling {}", self.name());
        let time = time::precise_time_ns();
        let mut current = HashMap::new();

        trace!("accessing data in kernelspace");
        let mut data = self.bpf.table("dist");

        trace!("copying data to userspace");
        for entry in data.iter() {
            let mut key = entry.key;
            let value = entry.value;

            // key is a u64 index into a BPF_HISTOGRAM
            let mut k = [0_u8; 4];
            for (index, byte) in k.iter_mut().enumerate() {
                *byte = *key.get(index).unwrap_or(&0);
            }
            let k: u32 = unsafe { mem::transmute(k) };

            // convert the key to a block size in kbytes
            if let Some(latency) = super::key_to_value(k as u64) {
                let latency = latency * MICROSECOND;
                // value is a u64 count of times that block size was seen
                let mut v = [0_u8; 8];
                for (index, byte) in v.iter_mut().enumerate() {
                    *byte = *value.get(index).unwrap_or(&0);
                }

                let count: u64 = unsafe { mem::transmute(v) };

                // store the size-count pair into the hashmap
                current.insert(latency, count as u32);
            }

            // clear the source counter
            let _ = data.set(&mut key, &mut [0_u8; 8]);
        }
        trace!("data copied to userspace");
        if !self.initialized {
            self.register(recorder, config);
        } else {
            for (&latency, &count) in &current {
                record_distribution(
                    recorder,
                    "scheduler/runqueue_latency_ns",
                    time,
                    latency,
                    count,
                );
            }
        }

        Ok(())
    }

    fn register(&mut self, recorder: &Recorder<u32>, config: &Config) {
        debug!("register {}", self.name());
        if !self.initialized {
            register_distribution(
                recorder,
                "scheduler/runqueue_latency_ns",
                SECOND,
                2,
                config.general().interval(),
                PERCENTILES,
            );
            self.initialized = true;
        }
    }

    fn deregister(&mut self, recorder: &Recorder<u32>, _config: &Config) {
        debug!("deregister {}", self.name());
        if self.initialized {
            recorder.delete_channel("scheduler/runqueue_latency_ns".to_string());
            self.initialized = false;
        }
    }
}
