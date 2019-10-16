// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub(crate) mod statistics;

use self::statistics::*;
use crate::common::file::file_as_u64;
use crate::config::*;
use crate::samplers::{Common, Sampler};

use failure::Error;
use logger::*;
use metrics::*;
use time;

use std::sync::Arc;

pub struct CpuIdle {
    common: Common,
    cores: u64,
}

// Get CPU idle usage for a given state and CPU, in nanoseconds.
fn get_cpuidle_usage(cpu: u64, state: &Statistic) -> Result<u64, ()> {
    let filename = format!("/sys/devices/system/cpu/cpu{}/{}/usage", cpu, state);
    file_as_u64(filename).map(|x| x * 1000)
}

impl Sampler for CpuIdle {
    fn new(
        config: Arc<Config>,
        metrics: Metrics<AtomicU32>,
    ) -> Result<Option<Box<dyn Sampler>>, Error> {
        if config.cpuidle().enabled() {
            Ok(Some(Box::new(CpuIdle {
                common: Common::new(config, metrics),
                cores: crate::common::hardware_threads().unwrap_or(1),
            }) as Box<dyn Sampler>))
        } else {
            Ok(None)
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn name(&self) -> String {
        "cpuidle".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        trace!("sample {}", self.name());
        let time = time::precise_time_ns();
        self.register();
        for statistic in self.common.config().cpuidle().statistics() {
            let mut sum: u64 = 0;
            for core in 0..self.cores {
                sum += get_cpuidle_usage(core, &statistic)?;
            }
            self.common.record_counter(&statistic, time, sum);
        }
        Ok(())
    }

    fn interval(&self) -> usize {
        self.common()
            .config()
            .cpuidle()
            .interval()
            .unwrap_or_else(|| self.common().config().interval())
    }

    fn register(&mut self) {
        trace!("register {}", self.name());
        if !self.common.initialized() {
            for statistic in self.common.config().cpuidle().statistics() {
                self.common.register_counter(&statistic, 0, 0, &[]);
            }
            self.common.set_initialized(true);
        }
    }

    fn deregister(&mut self) {
        trace!("deregister {}", self.name());
        for statistic in self.common.config().cpu().statistics() {
            self.common.delete_channel(&statistic);
        }
        self.common.set_initialized(false);
    }
}
