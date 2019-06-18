// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod event;

pub use self::event::Event;
use crate::stats::{record_counter, register_counter};

use crate::common::*;
use crate::config::Config;
use crate::sampler::{Sampler, SamplerError};

use logger::*;
use metrics::*;
use perfcnt::AbstractPerfCounter;
use perfcnt::PerfCounter;
use time;

use std::collections::HashMap;

pub struct Perf {
    counters: HashMap<Event, Vec<PerfCounter>>,
    initialized: bool,
}

impl Perf {
    pub fn new(config: &Config) -> Self {
        let mut enabled = Vec::new();
        for event in config.perf().events() {
            if let Ok(event) = Event::from_str(&event) {
                enabled.push(event);
            }
        }

        let mut counters = HashMap::new();
        let cores = hardware_threads().unwrap_or(1);

        for event in &enabled {
            let mut event_counters = Vec::new();
            for core in 0..cores {
                match event
                    .builder()
                    .on_cpu(core as isize)
                    .for_all_pids()
                    .finish()
                {
                    Ok(c) => event_counters.push(c),
                    Err(e) => {
                        debug!("Failed to create PerfCounter for {:?}: {}", event, e);
                        break;
                    }
                }
            }
            if event_counters.len() as u64 == cores {
                trace!("Initialized PerfCounters for {:?}", event);
                counters.insert(*event, event_counters);
            }
        }

        Self {
            counters,
            initialized: false,
        }
    }
}

impl Sampler for Perf {
    fn name(&self) -> String {
        "perf".to_string()
    }

    fn sample(&mut self, recorder: &Recorder<u32>, config: &Config) -> Result<(), SamplerError> {
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
        if !self.initialized {
            self.register(recorder, config);
        }
        for event in self.counters.keys() {
            if let Some(counter) = current.get(event) {
                let value: u64 = counter.iter().sum();
                record_counter(recorder, format!("perf/{}", event), time, value);
            }
        }
        Ok(())
    }

    fn register(&mut self, recorder: &Recorder<u32>, config: &Config) {
        trace!("register {}", self.name());
        if !self.initialized {
            for event in self.counters.keys() {
                register_counter(
                    recorder,
                    format!("perf/{}", event),
                    TRILLION,
                    3,
                    config.general().interval(),
                    PERCENTILES,
                );
            }
            self.initialized = true;
        }
    }

    fn deregister(&mut self, recorder: &Recorder<u32>, _config: &Config) {
        trace!("deregister {}", self.name());
        if self.initialized {
            for event in self.counters.keys() {
                recorder.delete_channel(format!("perf/{}", event));
            }
            self.initialized = false;
        }
    }
}
