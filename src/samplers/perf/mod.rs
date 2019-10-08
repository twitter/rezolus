// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub mod statistics;

use self::statistics::*;
use crate::common::*;
use crate::config::*;
use crate::samplers::{Common, Sampler};

use failure::Error;
use logger::*;
use metrics::*;
use perfcnt::{AbstractPerfCounter, PerfCounter};
use time;

use std::collections::HashMap;

pub struct Perf<'a> {
    common: Common<'a>,
    counters: HashMap<Statistic, Vec<PerfCounter>>,
}

impl<'a> Sampler<'a> for Perf<'a> {
    fn new(
        config: &'a Config,
        metrics: &'a Metrics<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
        if config.perf().enabled() {
            let mut counters = HashMap::new();
            let cores = hardware_threads().unwrap_or(1);

            for statistic in config.perf().statistics() {
                let mut event_counters = Vec::new();
                for core in 0..cores {
                    match statistic
                        .builder()
                        .on_cpu(core as isize)
                        .for_all_pids()
                        .finish()
                    {
                        Ok(c) => event_counters.push(c),
                        Err(e) => {
                            debug!("Failed to create PerfCounter for {:?}: {}", statistic, e);
                            break;
                        }
                    }
                }
                if event_counters.len() as u64 == cores {
                    trace!("Initialized PerfCounters for {:?}", statistic);
                    counters.insert(*statistic, event_counters);
                }
            }

            Ok(Some(Box::new(Self {
                common: Common::new(config, metrics),
                counters,
            })))
        } else {
            Ok(None)
        }
    }

    fn common(&self) -> &Common<'a> {
        &self.common
    }

    fn name(&self) -> String {
        "perf".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        trace!("sample {}", self.name());
        let time = time::precise_time_ns();
        let mut current = HashMap::new();
        trace!("sampling: {} perf counters", self.counters.keys().count());
        for (event, counters) in &mut self.counters {
            let mut c = Vec::new();
            for counter in counters {
                let count = match counter.read() {
                    Ok(c) => c,
                    Err(e) => {
                        debug!("Could not read perf counter for event {:?}: {}", event, e);
                        0
                    }
                };
                c.push(count);
            }
            current.insert(*event, c);
        }
        self.register();
        for statistic in self.counters.keys() {
            if let Some(counter) = current.get(statistic) {
                let value: u64 = counter.iter().sum();
                self.common.record_counter(&statistic, time, value);
            }
        }
        Ok(())
    }

    fn interval(&self) -> usize {
        self.common()
            .config()
            .perf()
            .interval()
            .unwrap_or_else(|| self.common().config().interval())
    }

    fn register(&mut self) {
        if !self.common.initialized() {
            trace!("register {}", self.name());
            for statistic in self.counters.keys() {
                self.common
                    .register_counter(statistic, TRILLION, 3, PERCENTILES);
            }
            self.common.set_initialized(true);
        }
    }

    fn deregister(&mut self) {
        if self.common.initialized() {
            trace!("deregister {}", self.name());
            for statistic in self.counters.keys() {
                self.common.delete_channel(statistic);
            }
            self.common.set_initialized(false);
        }
    }
}
