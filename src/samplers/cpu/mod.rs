// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod config;
mod stat;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chashmap::CHashMap;
use metrics::*;
#[cfg(feature = "perf")]
use perfcnt::*;
use regex::Regex;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::prelude::*;
use tokio::runtime::Handle;

use crate::common::*;
use crate::config::{Config, SamplerConfig};
use crate::samplers::Common;
use crate::Sampler;

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
    SECOND / ticks_per_second
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
                let cores = crate::common::hardware_threads().unwrap_or(1024);
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
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }

        debug!("sampling");
        self.register();

        self.sample_cstates().await?;
        self.sample_cpu_usage().await?;
        #[cfg(feature = "perf")]
        self.sample_perf_counters().await?;

        Ok(())
    }

    fn summary(&self, _statistic: &Self::Statistic) -> Option<Summary> {
        let max = crate::common::hardware_threads().unwrap_or(1024) * SECOND;
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

    async fn sample_cstates(&self) -> Result<(), std::io::Error> {
        let mut result = HashMap::<CpuStatistic, u64>::new();

        // iterate through all cpus
        let cpu_regex = Regex::new(r"^cpu\d+$").unwrap();
        let state_regex = Regex::new(r"^state\d+$").unwrap();
        let mut cpu_dir = tokio::fs::read_dir("/sys/devices/system/cpu").await?;
        while let Some(cpu_entry) = cpu_dir.next_entry().await? {
            if let Ok(cpu_name) = cpu_entry.file_name().into_string() {
                if cpu_regex.is_match(&cpu_name) {
                    // iterate through all cpuidle states
                    let cpuidle_dir = format!("/sys/devices/system/cpu/{}/cpuidle", cpu_name);
                    let mut cpuidle_dir = tokio::fs::read_dir(cpuidle_dir).await?;
                    while let Some(cpuidle_entry) = cpuidle_dir.next_entry().await? {
                        if let Ok(cpuidle_name) = cpuidle_entry.file_name().into_string() {
                            if state_regex.is_match(&cpuidle_name) {
                                // have an actual state here

                                // get the name of the state
                                let name_file = format!(
                                    "/sys/devices/system/cpu/{}/cpuidle/{}/name",
                                    cpu_name, cpuidle_name
                                );
                                let mut name_file = File::open(name_file).await?;
                                let mut name_content = Vec::new();
                                name_file.read_to_end(&mut name_content).await?;
                                if let Ok(name_string) = std::str::from_utf8(&name_content) {
                                    if let Ok(state) = name_string.parse() {
                                        // get the time spent in the state
                                        let time_file = format!(
                                            "/sys/devices/system/cpu/{}/cpuidle/{}/time",
                                            cpu_name, cpuidle_name
                                        );
                                        let mut time_file = File::open(time_file).await?;
                                        let mut time_content = Vec::new();
                                        time_file.read_to_end(&mut time_content).await?;
                                        if let Ok(time_string) = std::str::from_utf8(&time_content)
                                        {
                                            if let Ok(time) = time_string.parse::<u64>() {
                                                let metric = match state {
                                                    CState::C0 => CpuStatistic::CstateC0Time,
                                                    CState::C1 => CpuStatistic::CstateC1Time,
                                                    CState::C1E => CpuStatistic::CstateC1ETime,
                                                    CState::C2 => CpuStatistic::CstateC2Time,
                                                    CState::C3 => CpuStatistic::CstateC3Time,
                                                    CState::C6 => CpuStatistic::CstateC6Time,
                                                    CState::C7 => CpuStatistic::CstateC7Time,
                                                    CState::C8 => CpuStatistic::CstateC8Time,
                                                };
                                                let counter = result.entry(metric).or_insert(0);
                                                *counter += time;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let time = time::precise_time_ns();
        for stat in self.sampler_config().statistics() {
            if let Some(value) = result.get(stat) {
                self.metrics().record_counter(stat, time, *value);
            }
        }

        Ok(())
    }
}
