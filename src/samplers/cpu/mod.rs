// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::{HashMap, HashSet};
use std::io::SeekFrom;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::*;

use async_trait::async_trait;
#[cfg(feature = "bpf")]
use bcc::perf_event::{Event, SoftwareEvent};
#[cfg(feature = "bpf")]
use bcc::{PerfEvent, PerfEventArray};
use regex::Regex;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, BufReader};

use crate::common::bpf::BPF;
use crate::common::*;
use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::Sampler;

mod config;
mod stat;

pub use config::*;
pub use stat::*;
use std::iter::FromIterator;

#[allow(dead_code)]
pub struct Cpu {
    common: Common,
    cpus: HashSet<String>,
    cstates: HashMap<String, String>,
    cstate_files: HashMap<String, HashMap<String, File>>,
    perf: Option<Arc<Mutex<BPF>>>,
    tick_duration: u64,
    proc_cpuinfo: Option<File>,
    proc_stat: Option<File>,
    statistics: Vec<CpuStatistic>,
    stats: CpuStats,
}

pub fn nanos_per_tick() -> u64 {
    let ticks_per_second = sysconf::raw::sysconf(sysconf::raw::SysconfVariable::ScClkTck)
        .expect("Failed to get Clock Ticks per Second") as u64;
    SECOND / ticks_per_second
}

#[async_trait]
impl Sampler for Cpu {
    type Statistic = CpuStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let statistics = common.config().samplers().cpu().statistics();
        #[allow(unused_mut)]
        let mut sampler = Self {
            stats: CpuStats::new(&common, common.config.samplers().cpu().percentiles()),
            common,
            cpus: HashSet::new(),
            cstates: HashMap::new(),
            cstate_files: HashMap::new(),
            perf: None,
            tick_duration: nanos_per_tick(),
            proc_cpuinfo: None,
            proc_stat: None,
            statistics,
        };

        if sampler.sampler_config().enabled() {
            sampler.register();
        }

        // we initialize perf last so we can delay
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

        sampler
            .stats
            .disable_unwanted(&HashSet::from_iter(sampler.statistics.iter().copied()));

        // delay by half the sample interval so that we land between perf
        // counter updates
        std::thread::sleep(std::time::Duration::from_micros(
            (1000 * sampler.interval()) as u64 / 2,
        ));

        Ok(sampler)
    }

    fn spawn(common: Common) {
        if common.config().samplers().cpu().enabled() {
            if let Ok(mut cpu) = Cpu::new(common.clone()) {
                common.runtime().spawn(async move {
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

        // we do perf sampling first, since it is time critical to keep it
        // between underlying counter updates
        #[cfg(feature = "bpf")]
        {
            let r = self.sample_bpf_perf_counters();
            self.map_result(r)?;
        }

        let r = self.sample_cpuinfo().await;
        self.map_result(r)?;

        let r = self.sample_cpu_usage().await;
        self.map_result(r)?;

        let r = self.sample_cstates().await;
        self.map_result(r)?;

        Ok(())
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
            for statistic in &self.statistics {
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
            debug!("attaching software event to drive perf counter sampling");
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

    async fn sample_cpu_usage(&mut self) -> Result<(), std::io::Error> {
        if self.proc_stat.is_none() {
            let file = File::open("/proc/stat").await?;
            self.proc_stat = Some(file);
        }

        if let Some(file) = &mut self.proc_stat {
            file.seek(SeekFrom::Start(0)).await?;
            let time = Instant::now();

            let mut reader = BufReader::new(file);
            let mut buf = String::new();
            while reader.read_line(&mut buf).await? > 0 {
                Self::record_proc_stat(&self.stats, time, &buf);
                buf.clear();
            }
        }

        Ok(())
    }

    async fn sample_cpuinfo(&mut self) -> Result<(), std::io::Error> {
        if self.proc_cpuinfo.is_none() {
            let file = File::open("/proc/cpuinfo").await?;
            self.proc_cpuinfo = Some(file);
        }

        if let Some(file) = &mut self.proc_cpuinfo {
            file.seek(SeekFrom::Start(0)).await?;
            let mut reader = BufReader::new(file);
            let mut buf = String::new();
            let mut result = Vec::new();
            while reader.read_line(&mut buf).await? > 0 {
                if let Some(freq) = parse_frequency(&buf) {
                    result.push(freq.ceil() as i64);
                }
                buf.clear();
            }

            let time = Instant::now();
            for frequency in result {
                self.stats.frequency.store(time, frequency);
            }
        }

        Ok(())
    }

    #[cfg(feature = "bpf")]
    fn sample_bpf_perf_counters(&self) -> Result<(), std::io::Error> {
        use self::CpuStatistic::*;

        if let Some(ref bpf) = self.perf {
            let bpf = bpf.lock().unwrap();
            let time = Instant::now();

            let counter_value = |stat: CpuStatistic| -> Option<u64> {
                let table = bpf.inner.table(stat.table()?).ok()?;
                let map = crate::common::bpf::perf_table_to_map(&table);
                Some(map.iter().map(|(_cpu, count)| *count).sum())
            };

            if_block! {
                if let Some(value) = counter_value(BpuBranches) => self.stats.bpu_branches.store(time, value);
                if let Some(value) = counter_value(BpuMiss) => self.stats.bpu_miss.store(time, value);
                if let Some(value) = counter_value(CacheMiss) => self.stats.cache_miss.store(time, value);
                if let Some(value) = counter_value(Cycles) => self.stats.cycles.store(time, value);
                if let Some(value) = counter_value(DtlbLoadMiss) => self.stats.dtlb_load_miss.store(time, value);
                if let Some(value) = counter_value(DtlbLoadAccess) => self.stats.dtlb_load_access.store(time, value);
                if let Some(value) = counter_value(DtlbStoreMiss) => self.stats.dtlb_store_miss.store(time, value);
                if let Some(value) = counter_value(DtlbStoreAccess) => self.stats.dtlb_store_access.store(time, value);
                if let Some(value) = counter_value(Instructions) => self.stats.instructions.store(time, value);
                if let Some(value) = counter_value(ReferenceCycles) => self.stats.reference_cycles.store(time, value);
            }
        }
        Ok(())
    }

    async fn sample_cstates(&mut self) -> Result<(), std::io::Error> {
        use self::CpuStatistic::*;

        let mut result = HashMap::<CpuStatistic, u64>::new();

        // populate the cpu cache if empty
        if self.cpus.is_empty() {
            let cpu_regex = Regex::new(r"^cpu\d+$").unwrap();
            let mut cpu_dir = tokio::fs::read_dir("/sys/devices/system/cpu").await?;
            while let Some(cpu_entry) = cpu_dir.next_entry().await? {
                if let Ok(cpu_name) = cpu_entry.file_name().into_string() {
                    if cpu_regex.is_match(&cpu_name) {
                        self.cpus.insert(cpu_name.to_string());
                    }
                }
            }
        }

        // populate the cstate cache if empty
        if self.cstates.is_empty() {
            let state_regex = Regex::new(r"^state\d+$").unwrap();
            for cpu in &self.cpus {
                // iterate through all cpuidle states
                let cpuidle_dir = format!("/sys/devices/system/cpu/{}/cpuidle", cpu);
                let mut cpuidle_dir = tokio::fs::read_dir(cpuidle_dir).await?;
                while let Some(cpuidle_entry) = cpuidle_dir.next_entry().await? {
                    if let Ok(cpuidle_name) = cpuidle_entry.file_name().into_string() {
                        if state_regex.is_match(&cpuidle_name) {
                            // get the name of the state
                            let name_file = format!(
                                "/sys/devices/system/cpu/{}/cpuidle/{}/name",
                                cpu, cpuidle_name
                            );
                            let mut name_file = File::open(name_file).await?;
                            let mut name_content = Vec::new();
                            name_file.read_to_end(&mut name_content).await?;
                            if let Ok(name_string) = std::str::from_utf8(&name_content) {
                                if let Some(Ok(state)) =
                                    name_string.split_whitespace().next().map(|v| v.parse())
                                {
                                    self.cstates.insert(cpuidle_name, state);
                                }
                            }
                        }
                    }
                }
            }
        }

        for cpu in &self.cpus {
            if !self.cstate_files.contains_key(cpu) {
                self.cstate_files.insert(cpu.to_string(), HashMap::new());
            }
            if let Some(cpuidle_files) = self.cstate_files.get_mut(cpu) {
                for (cpuidle_name, state) in &self.cstates {
                    if !cpuidle_files.contains_key(cpuidle_name) {
                        let time_file = format!(
                            "/sys/devices/system/cpu/{}/cpuidle/{}/time",
                            cpu, cpuidle_name
                        );
                        let file = File::open(time_file).await?;
                        cpuidle_files.insert(cpuidle_name.to_string(), file);
                    }
                    if let Some(file) = cpuidle_files.get_mut(cpuidle_name) {
                        file.seek(SeekFrom::Start(0)).await?;
                        let mut reader = BufReader::new(file);
                        if let Ok(time) = reader.read_u64().await {
                            if let Some(state) = state.split('-').next() {
                                let metric = match CState::from_str(&state) {
                                    Ok(CState::C0) => CpuStatistic::CstateC0Time,
                                    Ok(CState::C1) => CpuStatistic::CstateC1Time,
                                    Ok(CState::C1E) => CpuStatistic::CstateC1ETime,
                                    Ok(CState::C2) => CpuStatistic::CstateC2Time,
                                    Ok(CState::C3) => CpuStatistic::CstateC3Time,
                                    Ok(CState::C6) => CpuStatistic::CstateC6Time,
                                    Ok(CState::C7) => CpuStatistic::CstateC7Time,
                                    Ok(CState::C8) => CpuStatistic::CstateC8Time,
                                    _ => continue,
                                };
                                let counter = result.entry(metric).or_insert(0);
                                *counter += time * MICROSECOND;
                            }
                        }
                    }
                }
            }
        }

        let time = Instant::now();
        if_block! {
            if let Some(&value) = result.get(&CstateC0Time) => self.stats.cstate_c0_time.store(time, value);
            if let Some(&value) = result.get(&CstateC1Time) => self.stats.cstate_c1_time.store(time, value);
            if let Some(&value) = result.get(&CstateC1ETime) => self.stats.cstate_c1e_time.store(time, value);
            if let Some(&value) = result.get(&CstateC2Time) => self.stats.cstate_c2_time.store(time, value);
            if let Some(&value) = result.get(&CstateC3Time) => self.stats.cstate_c3_time.store(time, value);
            if let Some(&value) = result.get(&CstateC6Time) => self.stats.cstate_c6_time.store(time, value);
            if let Some(&value) = result.get(&CstateC7Time) => self.stats.cstate_c7_time.store(time, value);
            if let Some(&value) = result.get(&CstateC8Time) => self.stats.cstate_c8_time.store(time, value);
        }

        Ok(())
    }

    // Note: returns option to make the implementation easier
    fn record_proc_stat(stats: &CpuStats, time: Instant, line: &str) -> Option<()> {
        let mut iter = line.split_whitespace();

        if iter.next()? != "cpu" {
            return Some(());
        }

        stats
            .usage_user
            .store(time, iter.next()?.parse().unwrap_or(0));
        stats
            .usage_nice
            .store(time, iter.next()?.parse().unwrap_or(0));
        stats
            .usage_system
            .store(time, iter.next()?.parse().unwrap_or(0));
        stats
            .usage_idle
            .store(time, iter.next()?.parse().unwrap_or(0));
        stats
            .usage_irq
            .store(time, iter.next()?.parse().unwrap_or(0));
        stats
            .usage_softirq
            .store(time, iter.next()?.parse().unwrap_or(0));
        stats
            .usage_steal
            .store(time, iter.next()?.parse().unwrap_or(0));
        stats
            .usage_guest
            .store(time, iter.next()?.parse().unwrap_or(0));
        stats
            .usage_guest_nice
            .store(time, iter.next()?.parse().unwrap_or(0));

        Some(())
    }
}

fn parse_frequency(line: &str) -> Option<f64> {
    let mut split = line.split_whitespace();
    if split.next() == Some("cpu") && split.next() == Some("MHz") {
        split.last().map(|v| v.parse().unwrap_or(0.0) * 1_000_000.0)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_frequency() {
        let result = parse_frequency("cpu MHz         : 1979.685");
        assert_eq!(result, Some(1_979_685_000.0));
    }
}
