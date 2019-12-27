// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::{Config, SamplerConfig};
use crate::samplers::{Common, Sampler};
use async_trait::async_trait;
use chashmap::CHashMap;
use metrics::*;
use perfcnt::{AbstractPerfCounter, PerfCounter};
use std::sync::Arc;
use tokio::runtime::Handle;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

pub struct Perf {
    common: Common,
    counters: CHashMap<PerfStatistic, Vec<PerfCounter>>,
}

#[async_trait]
impl Sampler for Perf {
    type Statistic = PerfStatistic;
    fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Result<Self, failure::Error> {
        let counters = CHashMap::new();
        let cores = 1;
        for statistic in config.perf().statistics().iter() {
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
                    }
                }
            }
            if event_counters.len() as u64 == cores {
                trace!("Initialized PerfCounters for {:?}", statistic);
                counters.insert(*statistic, event_counters);
            }
        }

        Ok(Self {
            common: Common::new(config, metrics),
            counters,
        })
    }

    fn spawn(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>, handle: &Handle) {
        if let Ok(mut sampler) = Self::new(config.clone(), metrics) {
            handle.spawn(async move {
                loop {
                    let _ = sampler.sample().await;
                }
            });
        } else if !config.fault_tolerant() {
            fatal!("failed to initialize sampler");
        } else {
            error!("failed to initialize sampler");
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().perf()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if !self.sampler_config().enabled() {
            if let Some(ref mut delay) = self.delay() {
                delay.tick().await;
            }

            return Ok(());
        }

        debug!("sampling");
        self.register();

        let time = time::precise_time_ns();
        for stat in self.sampler_config().statistics() {
            if let Some(mut counters) = self.counters.get_mut(stat) {
                let mut value = 0;
                for counter in counters.iter_mut() {
                    let count = match counter.read() {
                        Ok(c) => c,
                        Err(e) => {
                            debug!("Could not read perf counter for event {:?}: {}", stat, e);
                            0
                        }
                    };
                    value += count;
                }
                if value > 0 {
                    debug!("recording value for: {:?}", stat);
                }
                self.metrics().record_counter(stat, time, value);
            }
        }

        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        Ok(())
    }

    fn summary(&self, _statistic: &Self::Statistic) -> Option<Summary> {
        Some(Summary::histogram(
            1_000_000_000_000,
            3,
            Some(self.general_config().window()),
        ))
    }
}
