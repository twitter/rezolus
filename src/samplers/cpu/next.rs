#![allow(dead_code)]

use super::config::*;
use crate::common::bpf::BPF;
use crate::common::*;
use crate::config::SamplerConfig;
use crate::metrics::*;
use crate::samplers::Common;
use crate::Sampler;

use super::CpuStatistic;
use crate::samplers::cpu::CState;
use async_trait::async_trait;
#[cfg(feature = "bpf")]
use bcc::perf_event::{Event, SoftwareEvent};
#[cfg(feature = "bpf")]
use bcc::{PerfEvent, PerfEventArray};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::SeekFrom;
use std::iter::Cycle;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use strum_macros::{AsStaticStr, EnumIter, EnumString, IntoStaticStr};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, BufReader};
use std::str::FromStr;

macro_rules! stats_struct {
    {
        $( #[$attr:meta] )*
        $svis:vis struct $struct:ident : $stats:ident {
            $( $vis:vis $field:ident: $ty:ty = $name:literal ),* $(,)?
        }
    } => {
        $( #[$attr] )*
        $svis struct $struct{
            $( $vis $field: $ty, )*
        }

        impl $struct {
            #[allow(dead_code)]
            pub fn register(&mut self, enabled: &::std::collections::HashSet<$stats>) {
                $(
                    if enabled.contains(&$stats::$field) {
                        self.$field.register($name)
                    }
                )*
            }

            #[allow(dead_code)]
            pub fn with_config(config: &$crate::samplers::CommonSamplerConfig) -> Self {
                Self {
                    $( $field: <$ty>::with_config(config), )*
                }
            }
        }

        #[derive(
            Copy, Clone, Debug, Eq, PartialEq, Hash,
            Serialize, Deserialize, EnumIter, EnumString, IntoStaticStr,
            AsStaticStr,
        )]
        #[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
        #[allow(non_camel_case_types)]
        $svis enum $stats {
            $(
                #[strum(serialize = $name)]
                $field,
            )*
        }

        impl ::std::convert::TryFrom<&str> for $stats {
            type Error = ::strum::ParseError;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                use ::std::str::FromStr;

                Self::from_str(s)
            }
        }

        impl ::rustcommon_metrics::Statistic<
            ::rustcommon_atomics::AtomicU64,
            ::rustcommon_atomics::AtomicU32> for $stats {
            fn name(&self) -> &str {
                (*self).into()
            }

            // Not used but required
            fn source(&self) -> ::rustcommon_metrics::Source {
                unimplemented!()
            }
        }
    }
}

stats_struct! {
    pub(super) struct CpuStats : CpuStatKind {
        pub usage_user: StreamSummarizedCounter        = "cpu/usage/user",
        pub usage_nice: StreamSummarizedCounter        = "cpu/usage/nice",
        pub usage_system: StreamSummarizedCounter      = "cpu/usage/system",
        pub usage_idle: StreamSummarizedCounter        = "cpu/usage/idle",
        pub usage_irq: StreamSummarizedCounter         = "cpu/usage/irq",
        pub usage_softirq: StreamSummarizedCounter     = "cpu/usage/softirq",
        pub usage_steal: StreamSummarizedCounter       = "cpu/usage/steal",
        pub usage_guest: StreamSummarizedCounter       = "cpu/usage/guest",
        pub usage_guest_nice: StreamSummarizedCounter  = "cpu/usage/guestnice",
        pub cache_miss: StreamSummarizedCounter        = "cpu/cache/miss",
        pub cache_access: StreamSummarizedCounter      = "cpu/cache/access",
        pub bpu_branches: StreamSummarizedCounter      = "cpu/bpu/branch",
        pub bpu_miss: StreamSummarizedCounter          = "cpu/bpu/miss",
        pub cycles: StreamSummarizedCounter            = "cpu/cycles",
        pub dtlb_load_miss: StreamSummarizedCounter    = "cpu/dtlb/load/miss",
        pub dtlb_load_access: StreamSummarizedCounter  = "cpu/dtlb/load/access",
        pub dtlb_store_access: StreamSummarizedCounter = "cpu/dtlb/store/access",
        pub dtlb_store_miss: StreamSummarizedCounter   = "cpu/dtlb/store/miss",
        pub instructions: StreamSummarizedCounter      = "cpu/instructions",
        pub reference_cycles: StreamSummarizedCounter  = "cpu/reference_cycles",
        pub cstate_c0_time: StreamSummarizedCounter    = "cpu/cstate/c0/time",
        pub cstate_c1_time: StreamSummarizedCounter    = "cpu/cstate/c1/time",
        pub cstate_c1e_time: StreamSummarizedCounter   = "cpu/cstate/c1e/time",
        pub cstate_c2_time: StreamSummarizedCounter    = "cpu/cstate/c2/time",
        pub cstate_c3_time: StreamSummarizedCounter    = "cpu/cstate/c3/time",
        pub cstate_c6_time: StreamSummarizedCounter    = "cpu/cstate/c6/time",
        pub cstate_c7_time: StreamSummarizedCounter    = "cpu/cstate/c7/time",
        pub cstate_c8_time: StreamSummarizedCounter    = "cpu/cstate/c8/time",
        pub frequency: StreamSummarizedGauge           = "cpu/frequency",
    }
}

pub struct CpuSampler {
    common: Common,
    stats: CpuStats,

    cpus: HashSet<String>,
    cstates: HashMap<String, String>,
    cstate_files: HashMap<String, HashMap<String, File>>,
    perf: Option<Arc<Mutex<BPF>>>,
    tick_duration: u64,
    proc_cpuinfo: Option<File>,
    proc_stat: Option<File>,
}

// Note: not needed as part of the design, just for interoperability at the moment
impl From<CpuStatistic> for CpuStatKind {
    fn from(stat: CpuStatistic) -> Self {
        use self::CpuStatistic::*;

        match stat {
            UsageUser => CpuStatKind::usage_user,
            UsageNice => CpuStatKind::usage_nice,
            UsageSystem => CpuStatKind::usage_system,
            UsageIdle => CpuStatKind::usage_idle,
            UsageIrq => CpuStatKind::usage_irq,
            UsageSoftirq => CpuStatKind::usage_softirq,
            UsageSteal => CpuStatKind::usage_steal,
            UsageGuest => CpuStatKind::usage_guest,
            UsageGuestNice => CpuStatKind::usage_guest_nice,
            CacheMiss => CpuStatKind::cache_miss,
            CacheAccess => CpuStatKind::cache_access,
            BpuBranches => CpuStatKind::bpu_branches,
            BpuMiss => CpuStatKind::bpu_miss,
            Cycles => CpuStatKind::cycles,
            DtlbLoadMiss => CpuStatKind::dtlb_load_miss,
            DtlbLoadAccess => CpuStatKind::dtlb_load_access,
            DtlbStoreAccess => CpuStatKind::dtlb_store_access,
            DtlbStoreMiss => CpuStatKind::dtlb_store_miss,
            Instructions => CpuStatKind::instructions,
            ReferenceCycles => CpuStatKind::reference_cycles,
            CstateC0Time => CpuStatKind::cstate_c0_time,
            CstateC1Time => CpuStatKind::cstate_c1_time,
            CstateC1ETime => CpuStatKind::cstate_c1e_time,
            CstateC2Time => CpuStatKind::cstate_c2_time,
            CstateC3Time => CpuStatKind::cstate_c3_time,
            CstateC6Time => CpuStatKind::cstate_c6_time,
            CstateC7Time => CpuStatKind::cstate_c7_time,
            CstateC8Time => CpuStatKind::cstate_c8_time,
            Frequency => CpuStatKind::frequency,
        }
    }
}

pub fn nanos_per_tick() -> u64 {
    let ticks_per_second = sysconf::raw::sysconf(sysconf::raw::SysconfVariable::ScClkTck)
        .expect("Failed to get Clock Ticks per Second") as u64;
    SECOND / ticks_per_second
}

#[async_trait]
impl Sampler for CpuSampler {
    type Statistic = CpuStatKind;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let statistics: HashSet<_> = common
            .config()
            .samplers()
            .cpu()
            .statistics()
            .into_iter()
            .map(CpuStatKind::from)
            .collect();
        let config = common.common_sampler_config(common.config().samplers().cpu());

        #[allow(unused_mut)]
        let mut sampler = Self {
            common,
            stats: CpuStats::with_config(&config),
            cpus: HashSet::new(),
            cstates: HashMap::new(),
            cstate_files: HashMap::new(),
            perf: None,
            tick_duration: nanos_per_tick(),
            proc_cpuinfo: None,
            proc_stat: None,
        };

        if sampler.sampler_config().enabled() {
            sampler.stats.register(&statistics);

            // we initialize perf last so we can delay
            #[cfg(feature = "bpf")]
            if sampler.sampler_config().perf_events() {
                // if let Err(e) = sampler
            }
        }

        // delay by half the sample interval so that we land between perf
        // counter updates
        std::thread::sleep(std::time::Duration::from_micros(
            (1000 * sampler.interval()) as u64 / 2,
        ));

        Ok(sampler)
    }

    fn spawn(common: Common) {
        if common.config().samplers().cpu().enabled() {
            if let Ok(mut cpu) = Self::new(common.clone()) {
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

impl CpuSampler {
    #[cfg(feature = "bpf")]
    fn initialize_bpf_perf(
        &mut self,
        statistics: &HashSet<CpuStatKind>,
    ) -> Result<(), std::io::Error> {
        use bcc::perf_event::{CacheId, CacheOp, CacheResult, HardwareEvent};

        let cpus = crate::common::hardware_threads().unwrap();
        let interval = self.interval() as u64;
        let frequency = match interval {
            0 => 1,
            _ if interval > 1000 => 1,
            _ => 1000 / interval,
        };

        let code = format!(
            "{}\n{}",
            format!("#define NUM_CPU {}", cpus),
            include_str!("perf.c").to_string()
        );

        let mut bpf = match bcc::BPF::new(&code) {
            Ok(bpf) => bpf,
            Err(_) => {
                if !self.common().config().general().fault_tolerant() {
                    fatal!("failed to initialize perf bpf for cpu");
                } else {
                    error!("failed to initialize perf bpf for cpu");
                }
                return Ok(());
            }
        };

        let events = [
            (
                "branch_instructions",
                Event::Hardware(HardwareEvent::BranchInstructions),
            ),
            (
                "branch_misses",
                Event::Hardware(HardwareEvent::BranchMisses),
            ),
            (
                "cache_references",
                Event::Hardware(HardwareEvent::CacheReferences),
            ),
            ("cache_misses", Event::Hardware(HardwareEvent::CacheMisses)),
            ("cycles", Event::Hardware(HardwareEvent::CpuCycles)),
            (
                "dtlb_load_miss",
                Event::HardwareCache(CacheId::DTLB, CacheOp::Read, CacheResult::Miss),
            ),
            (
                "dtlb_load_access",
                Event::HardwareCache(CacheId::DTLB, CacheOp::Read, CacheResult::Access),
            ),
            (
                "dtlb_store_miss",
                Event::HardwareCache(CacheId::DTLB, CacheOp::Write, CacheResult::Miss),
            ),
            (
                "dtlb_store_access",
                Event::HardwareCache(CacheId::DTLB, CacheOp::Write, CacheResult::Access),
            ),
            ("instructions", Event::Hardware(HardwareEvent::Instructions)),
        ];

        for (table, event) in events {
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
            info!("freq: {:?}", result);
            for frequency in result {
                self.stats.frequency.store(time, frequency);
            }
        }

        Ok(())
    }

    #[cfg(feature = "bpf")]
    fn sample_bpf_perf_counters(&self) -> Result<(), std::io::Error> {
        let bpf = match self.perf {
            Some(ref bpf) => bpf,
            None => return Ok(()),
        };

        let bpf = bpf.lock().unwrap();
        let time = Instant::now();

        let counters = [
            ("branch_instructions", &self.stats.bpu_branches),
            ("branch_misses", &self.stats.bpu_miss),
            ("cache_misses", &self.stats.cache_miss),
            ("cache_access", &self.stats.cache_access),
            ("cycles", &self.stats.cycles),
            ("dtlb_load_miss", &self.stats.dtlb_load_miss),
            ("dtlb_load_access", &self.stats.dtlb_load_access),
            ("dtlb_store_miss", &self.stats.dtlb_store_miss),
            ("dtlb_store_access", &self.stats.dtlb_store_access),
            ("instructions", &self.stats.instructions),
            ("reference_cycles", &self.stats.reference_cycles),
        ];

        for (table, counter) in counters {
            let table = match bpf.inner.table(table) {
                Ok(table) => table,
                Err(_) => continue,
            };

            let map = crate::common::bpf::perf_table_to_map(&table);
            counter.store(time, map.iter().map(|(_, count)| count).sum());
        }

        Ok(())
    }

    async fn sample_cstates(&mut self) -> Result<(), std::io::Error> {
        let mut result = HashMap::<CState, u64>::new();

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

                    let file = match cpuidle_files.get_mut(cpuidle_name) {
                        Some(file) => file,
                        None => continue,
                    };

                    file.seek(SeekFrom::Start(0)).await?;
                    let mut reader = BufReader::new(file);

                    let time = match reader.read_u64().await {
                        Ok(time) => time,
                        Err(_) => continue,
                    };
                    let state = match state.split('-').next() {
                        Some(state) => state,
                        None => continue,
                    };
                    let cstate = match CState::from_str(state) {
                        Ok(cstate) => cstate,
                        _ => continue
                    };

                    *result.entry(cstate).or_insert(0) += time * MICROSECOND;
                }
            }
        }

        let time = Instant::now();
        let metrics = [
            (CState::C0, &self.stats.cstate_c0_time),
            (CState::C1, &self.stats.cstate_c1_time),
            (CState::C1E, &self.stats.cstate_c1e_time),
            (CState::C2, &self.stats.cstate_c2_time),
            (CState::C3, &self.stats.cstate_c3_time),
            (CState::C6, &self.stats.cstate_c6_time),
            (CState::C7, &self.stats.cstate_c7_time),
            (CState::C8, &self.stats.cstate_c8_time)
        ];

        for (cstate, metric) in metrics {
            if let Some(&value) = result.get(&cstate) {
                metric.store(time, value);
            }
        }

        Ok(())
    }

    // Note: returns option to make the implementation easier
    fn record_proc_stat(stats: &CpuStats, time: Instant, line: &str) -> Option<()> {
        let mut iter = line.split_whitespace();

        if iter.next()? != "cpu" {
            return Some(());
        }

        let stats = [
            &stats.usage_user,
            &stats.usage_nice,
            &stats.usage_system,
            &stats.usage_idle,
            &stats.usage_irq,
            &stats.usage_softirq,
            &stats.usage_steal,
            &stats.usage_guest,
            &stats.usage_guest_nice
        ];

        for (stat, value) in stats.into_iter().zip(iter) {
            stat.store(time, value.parse().unwrap_or(0));
        }

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
