// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "perf")]
use perfcnt::*;

use crate::config::{Config, SamplerConfig};
use crate::samplers::Common;
use crate::Sampler;
use async_trait::async_trait;
use chashmap::CHashMap;
use metrics::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::runtime::Handle;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[cfg(not(feature = "perf"))]
struct PerfCounter {}

#[allow(dead_code)]
pub struct Cpu {
    common: Common,
    perf_counters: CHashMap<CpuStatistic, Vec<PerfCounter>>,
    tick_duration: u64,
}

pub fn nanos_per_tick() -> u64 {
    let ticks_per_second = sysconf::raw::sysconf(sysconf::raw::SysconfVariable::ScClkTck)
        .expect("Failed to get Clock Ticks per Second") as u64;
    1_000_000_000 / ticks_per_second
}

impl Cpu {
    pub fn spawn(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>, handle: &Handle) {
        if let Ok(mut cpu) = Cpu::new(config.clone(), metrics) {
            handle.spawn(async move {
                loop {
                    let _ = cpu.sample().await;
                }
            });
        } else if !config.fault_tolerant() {
            fatal!("failed to initialize cpu sampler");
        } else {
            error!("failed to initialize cpu sampler");
        }
    }
}

#[async_trait]
impl Sampler for Cpu {
    type Statistic = CpuStatistic;

    fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Result<Self, failure::Error> {
        let perf_counters = CHashMap::new();
        if config.samplers().cpu().perf_events() {
            #[cfg(feature = "perf")]
            {
                // TODO: core detection
                let cores = 1;
                for statistic in config.samplers().cpu().statistics().iter() {
                    if let Some(mut builder) = statistic.perf_counter_builder() {
                        let mut event_counters = Vec::new();
                        for core in 0..cores {
                            match builder.on_cpu(core as isize).for_all_pids().finish() {
                                Ok(c) => event_counters.push(c),
                                Err(e) => {
                                    debug!(
                                        "Failed to create PerfCounter for {:?}: {}",
                                        statistic, e
                                    );
                                }
                            }
                        }
                        if event_counters.len() as u64 == cores {
                            trace!("Initialized PerfCounters for {:?}", statistic);
                            perf_counters.insert(*statistic, event_counters);
                        }
                    }
                }
            }
        }
        Ok(Self {
            common: Common::new(config, metrics),
            perf_counters,
            tick_duration: nanos_per_tick(),
        })
    }

    fn spawn(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>, handle: &Handle) {
        if let Ok(mut cpu) = Cpu::new(config.clone(), metrics) {
            handle.spawn(async move {
                loop {
                    let _ = cpu.sample().await;
                }
            });
        } else if !config.fault_tolerant() {
            fatal!("failed to initialize cpu sampler");
        } else {
            error!("failed to initialize cpu sampler");
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().cpu()
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

        self.sample_cpu_usage().await?;
        #[cfg(feature = "perf")]
        self.sample_perf_counters().await?;

        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        Ok(())
    }

    fn summary(&self, _statistic: &Self::Statistic) -> Option<Summary> {
        let max = crate::common::hardware_threads().unwrap_or(1024) * 2_000_000_000;
        Some(Summary::histogram(
            max,
            3,
            Some(self.general_config().window()),
        ))
    }
}

impl Cpu {
    async fn sample_cpu_usage(&self) -> Result<(), std::io::Error> {
        let file = File::open("/proc/stat").await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut result = HashMap::new();

        while let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts[0] == "cpu" && parts.len() == 11 {
                result.insert(CpuStatistic::UsageUser, parts[1].parse().unwrap_or(0));
                result.insert(CpuStatistic::UsageNice, parts[2].parse().unwrap_or(0));
                result.insert(CpuStatistic::UsageSystem, parts[3].parse().unwrap_or(0));
                result.insert(CpuStatistic::UsageIdle, parts[4].parse().unwrap_or(0));
                result.insert(CpuStatistic::UsageIrq, parts[6].parse().unwrap_or(0));
                result.insert(CpuStatistic::UsageSoftirq, parts[7].parse().unwrap_or(0));
                result.insert(CpuStatistic::UsageSteal, parts[8].parse().unwrap_or(0));
                result.insert(CpuStatistic::UsageGuest, parts[9].parse().unwrap_or(0));
                result.insert(CpuStatistic::UsageGuestNice, parts[10].parse().unwrap_or(0));
            }
        }

        let time = time::precise_time_ns();
        for stat in self.sampler_config().statistics() {
            if let Some(value) = result.get(stat) {
                self.metrics()
                    .record_counter(stat, time, value * self.tick_duration);
            }
        }

        Ok(())
    }

    #[cfg(feature = "perf")]
    async fn sample_perf_counters(&mut self) -> Result<(), std::io::Error> {
        let time = time::precise_time_ns();
        for stat in self.sampler_config().statistics() {
            if let Some(mut counters) = self.perf_counters.get_mut(stat) {
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

        Ok(())
    }
}
