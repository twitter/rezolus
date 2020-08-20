// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
#[cfg(feature = "bpf")]
use bcc::perf_event::{Event, SoftwareEvent};
#[cfg(feature = "bpf")]
use bcc::{PerfEvent, PerfEventArray};
use regex::Regex;
use rustcommon_metrics::*;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::prelude::*;

use crate::common::bpf::BPF;
use crate::common::*;
use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::Sampler;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Cpu {
    common: Common,
    perf: Option<Arc<Mutex<BPF>>>,
    tick_duration: u64,
}

pub fn nanos_per_tick() -> u64 {
    let ticks_per_second = sysconf::raw::sysconf(sysconf::raw::SysconfVariable::ScClkTck)
        .expect("Failed to get Clock Ticks per Second") as u64;
    SECOND / ticks_per_second
}

#[async_trait]
impl Sampler for Cpu {
    type Statistic = CpuStatistic;

    fn new(common: Common) -> Result<Self, failure::Error> {
        #[allow(unused_mut)]
        let mut sampler = Self {
            common,
            perf: None,
            tick_duration: nanos_per_tick(),
        };

        if sampler.sampler_config().enabled() && sampler.sampler_config().perf_events() {
            #[cfg(feature = "bpf")]
            {
                if let Err(e) = sampler.initialize_bpf_perf() {
                    if !sampler.common().config().general().fault_tolerant() {
                        return Err(format_err!("bpf perf init failure: {}", e));
                    }
                }
            }
        }

        Ok(sampler)
    }

    fn spawn(common: Common) {
        if let Ok(mut cpu) = Cpu::new(common.clone()) {
            common.handle.spawn(async move {
                loop {
                    let _ = cpu.sample().await;
                }
            });
        } else if !common.config.fault_tolerant() {
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

        self.map_result(self.sample_cpuinfo().await)?;
        self.map_result(self.sample_cstates().await)?;
        self.map_result(self.sample_cpu_usage().await)?;
        #[cfg(feature = "bpf")]
        {
            let result = self.sample_bpf_perf_counters();
            self.map_result(result)?;
        }

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
    #[cfg(feature = "bpf")]
    fn initialize_bpf_perf(&mut self) -> Result<(), std::io::Error> {
        let cpus = crate::common::hardware_threads().unwrap();
        let interval = self.interval() as u64;
        let frequency = if interval > 1000 {
            1
        } else if interval == 0 {
            1
        } else {
            1000 / interval
        };

        let code = format!(
            "{}\n{}",
            format!("#define NUM_CPU {}", cpus),
            include_str!("perf.c").to_string()
        );
        if let Ok(mut bpf) = bcc::BPF::new(&code) {
            for statistic in self.sampler_config().statistics().iter() {
                if let Some(table) = statistic.table() {
                    if let Some(event) = statistic.event() {
                        if PerfEventArray::new()
                            .table(&format!("{}_array", table))
                            .event(event)
                            .attach(&mut bpf)
                            .is_err()
                        {
                            if !self.common().config().general().fault_tolerant() {
                                fatal!("failed to initialize perf bpf for event: {:?}", event);
                            } else {
                                error!("failed to initialize perf bpf for event: {:?}", event);
                            }
                        }
                    }
                }
            }
            if PerfEvent::new()
                .handler("do_count")
                .event(Event::Software(SoftwareEvent::CpuClock))
                .sample_frequency(Some(frequency))
                .attach(&mut bpf)
                .is_err()
            {
                if !self.common().config().general().fault_tolerant() {
                    fatal!("failed to initialize perf bpf for cpu");
                } else {
                    error!("failed to initialize perf bpf for cpu");
                }
            }
            self.perf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
        } else if !self.common().config().general().fault_tolerant() {
            fatal!("failed to initialize perf bpf");
        } else {
            error!("failed to initialize perf bpf. skipping cpu perf telemetry");
        }
        Ok(())
    }

    async fn sample_cpu_usage(&self) -> Result<(), std::io::Error> {
        let file = File::open("/proc/stat").await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut result = HashMap::new();

        while let Some(line) = lines.next_line().await? {
            result.extend(parse_proc_stat(&line));
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

    async fn sample_cpuinfo(&self) -> Result<(), std::io::Error> {
        let frequency_re = Regex::new(r"^cpu MHz\s+:\s+([\d\.]+)$").unwrap();

        let file = File::open("/proc/cpuinfo").await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let time = time::precise_time_ns();
        while let Some(line) = lines.next_line().await? {
            if let Some(freq) = frequency_re.captures(&line).map(|m| m.get(1).unwrap()) {
                if let Ok(mhz) = freq.as_str().parse::<f64>() {
                    self.metrics().record_gauge(
                        &CpuStatistic::Frequency,
                        time,
                        (mhz * 1_000_000.0_f64) as u64,
                    );
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "bpf")]
    fn sample_bpf_perf_counters(&self) -> Result<(), std::io::Error> {
        if let Some(ref bpf) = self.perf {
            let bpf = bpf.lock().unwrap();
            let time = time::precise_time_ns();
            for stat in self.sampler_config().statistics() {
                if let Some(table) = stat.table() {
                    let map = crate::common::bpf::perf_table_to_map(&(*bpf).inner.table(table));
                    let mut total = 0;
                    for (_cpu, count) in map.iter() {
                        total += count;
                    }
                    self.metrics().record_counter(stat, time, total);
                }
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
                                    let name_parts: Vec<&str> =
                                        name_string.split_whitespace().collect();
                                    if let Some(Ok(state)) = name_parts.get(0).map(|v| v.parse()) {
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
                                            let time_parts: Vec<&str> =
                                                time_string.split_whitespace().collect();
                                            if let Some(Ok(time)) =
                                                time_parts.get(0).map(|v| v.parse::<u64>())
                                            {
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
                                                *counter += time * MICROSECOND;
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

fn parse_proc_stat(line: &str) -> HashMap<CpuStatistic, u64> {
    let mut result = HashMap::new();
    let parts: Vec<&str> = line.split_whitespace().collect();
    if let Some(&"cpu") = parts.get(0) {
        match parts.len() {
            11 => {
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
            _ => {
                debug!("parsed cpu line but got unexpected number of fields");
            }
        }
    }
    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_proc_stat() {
        let result = parse_proc_stat("cpu  131586 0 53564 8246483 35015 350665 4288 5632 0 0");
        assert_eq!(result.len(), 9);
        assert_eq!(result.get(&CpuStatistic::UsageUser), Some(&131586));
        assert_eq!(result.get(&CpuStatistic::UsageNice), Some(&0));
        assert_eq!(result.get(&CpuStatistic::UsageSystem), Some(&53564));
    }
}
