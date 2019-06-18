// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::{MILLION, PERCENTILES};
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

pub struct Block {
    bpf: BPF,
    initialized: bool,
}

impl Block {
    pub fn new() -> Result<Self, Error> {
        debug!("initializing");
        // load the code and compile
        let code = include_str!("io_size.c");
        let mut bpf = BPF::new(code)?;
        // load + attach kprobes!
        let trace_pid_start = bpf.load_kprobe("trace_pid_start")?;
        let do_count = bpf.load_kprobe("do_count")?;

        bpf.attach_kprobe("blk_account_io_start", trace_pid_start)?;
        bpf.attach_kprobe("blk_account_io_completion", do_count)?;

        Ok(Self {
            bpf,
            initialized: false,
        })
    }
}

impl Sampler for Block {
    fn name(&self) -> String {
        "ebpf::block".to_string()
    }

    fn sample(&mut self, recorder: &Recorder<u32>, config: &Config) -> Result<(), SamplerError> {
        // gather current state
        trace!("sampling {}", self.name());
        let time = time::precise_time_ns();
        let mut current = HashMap::new();

        let mut data = self.bpf.table("dist");

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
            if let Some(size) = super::key_to_value(k as u64) {
                // value is a u64 count of times that block size was seen
                let mut v = [0_u8; 8];
                for (index, byte) in v.iter_mut().enumerate() {
                    *byte = *value.get(index).unwrap_or(&0);
                }

                let count: u64 = unsafe { mem::transmute(v) };

                // store the size-count pair into the hashmap
                current.insert(size, count as u32);
            }

            // clear the source counter
            let _ = data.set(&mut key, &mut [0_u8; 8]);
        }
        if !self.initialized {
            self.register(recorder, config);
        } else {
            for (&value, &count) in &current {
                record_distribution(recorder, "block/io_size_kb", time, value, count);
            }
        }
        Ok(())
    }

    fn register(&mut self, recorder: &Recorder<u32>, config: &Config) {
        debug!("register {}", self.name());
        if !self.initialized {
            register_distribution(
                recorder,
                "block/io_size_kb",
                MILLION,
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
            recorder.delete_channel("block/io_size_kb".to_string());
            self.initialized = false;
        }
    }
}
