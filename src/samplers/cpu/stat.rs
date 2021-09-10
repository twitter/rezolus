// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

#[cfg(feature = "bpf")]
use bcc::perf_event::*;

use rustcommon_metrics::*;
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{AsStaticStr, EnumIter, EnumString, IntoStaticStr};

use crate::metrics::*;
use std::time::Duration;
use std::collections::HashSet;

pub(super) struct CpuStats {
    pub usage_user: HeatmapSummarizedCounter,
    pub usage_nice: HeatmapSummarizedCounter,
    pub usage_system: HeatmapSummarizedCounter,
    pub usage_idle: HeatmapSummarizedCounter,
    pub usage_irq: HeatmapSummarizedCounter,
    pub usage_softirq: HeatmapSummarizedCounter,
    pub usage_steal: HeatmapSummarizedCounter,
    pub usage_guest: HeatmapSummarizedCounter,
    pub usage_guest_nice: HeatmapSummarizedCounter,
    pub cache_miss: HeatmapSummarizedCounter,
    pub cache_access: HeatmapSummarizedCounter,
    pub bpu_branches: HeatmapSummarizedCounter,
    pub bpu_miss: HeatmapSummarizedCounter,
    pub cycles: HeatmapSummarizedCounter,
    pub dtlb_load_miss: HeatmapSummarizedCounter,
    pub dtlb_load_access: HeatmapSummarizedCounter,
    pub dtlb_store_access: HeatmapSummarizedCounter,
    pub dtlb_store_miss: HeatmapSummarizedCounter,
    pub instructions: HeatmapSummarizedCounter,
    pub reference_cycles: HeatmapSummarizedCounter,
    pub cstate_c0_time: HeatmapSummarizedCounter,
    pub cstate_c1_time: HeatmapSummarizedCounter,
    pub cstate_c1e_time: HeatmapSummarizedCounter,
    pub cstate_c2_time: HeatmapSummarizedCounter,
    pub cstate_c3_time: HeatmapSummarizedCounter,
    pub cstate_c6_time: HeatmapSummarizedCounter,
    pub cstate_c7_time: HeatmapSummarizedCounter,
    pub cstate_c8_time: HeatmapSummarizedCounter,
    pub frequency: HeatmapSummarizedGauge,
}

impl CpuStats {
    pub fn new(common: &crate::samplers::Common, percentiles: &[f64]) -> Self {
        let span = Duration::from_secs(common.config().general().window() as _);
        let heatmap = || HeatmapSummarizedCounter::new(span, percentiles);

        Self {
            usage_user: heatmap(),
            usage_nice: heatmap(),
            usage_system: heatmap(),
            usage_idle: heatmap(),
            usage_irq: heatmap(),
            usage_softirq: heatmap(),
            usage_steal: heatmap(),
            usage_guest: heatmap(),
            usage_guest_nice: heatmap(),
            cache_miss: heatmap(),
            cache_access: heatmap(),
            bpu_branches: heatmap(),
            bpu_miss: heatmap(),
            cycles: heatmap(),
            dtlb_load_miss: heatmap(),
            dtlb_load_access: heatmap(),
            dtlb_store_access: heatmap(),
            dtlb_store_miss: heatmap(),
            instructions: heatmap(),
            reference_cycles: heatmap(),
            cstate_c0_time: heatmap(),
            cstate_c1_time: heatmap(),
            cstate_c1e_time: heatmap(),
            cstate_c2_time: heatmap(),
            cstate_c3_time: heatmap(),
            cstate_c6_time: heatmap(),
            cstate_c7_time: heatmap(),
            cstate_c8_time: heatmap(),
            frequency: HeatmapSummarizedGauge::new(span, percentiles),
        }
    }

    pub fn register(&mut self, stats: &HashSet<CpuStatistic>) {
        use self::CpuStatistic::*;

        if_block! {
            if stats.contains(&UsageUser) => self.usage_user.register("cpu/usage/user");
            if stats.contains(&UsageNice) => self.usage_nice.register("cpu/usage/nice");
            if stats.contains(&UsageSystem) => self.usage_system.register("cpu/usage/system");
            if stats.contains(&UsageIdle) => self.usage_idle.register("cpu/usage/idle");
            if stats.contains(&UsageIrq) => self.usage_irq.register("cpu/usage/irq");
            if stats.contains(&UsageSoftirq) => self.usage_softirq.register("cpu/usage/softirq");
            if stats.contains(&UsageSteal) => self.usage_steal.register("cpu/usage/steal");
            if stats.contains(&UsageGuest) => self.usage_guest.register("cpu/usage/guest");
            if stats.contains(&UsageGuestNice) => self.usage_guest_nice.register("cpu/usage/guestnice");
            if stats.contains(&CacheMiss) => self.cache_miss.register("cpu/cache/miss");
            if stats.contains(&CacheAccess) => self.cache_access.register("cpu/cache/access");
            if stats.contains(&BpuBranches) => self.bpu_branches.register("cpu/bpu/branch");
            if stats.contains(&BpuMiss) => self.bpu_miss.register("cpu/bpu/miss");
            if stats.contains(&Cycles) => self.cycles.register("cpu/cycles");
            if stats.contains(&DtlbLoadMiss) => self.dtlb_load_miss.register("cpu/dtlb/load/miss");
            if stats.contains(&DtlbLoadAccess) => self.dtlb_load_access.register("cpu/dtlb/load/access");
            if stats.contains(&DtlbStoreAccess) => self.dtlb_store_access.register("cpu/dtlb/store/access");
            if stats.contains(&DtlbStoreMiss) => self.dtlb_store_miss.register("cpu/dtlb/store/miss");
            if stats.contains(&Instructions) => self.instructions.register("cpu/instructions");
            if stats.contains(&ReferenceCycles) => self.reference_cycles.register("cpu/reference_cycles");
            if stats.contains(&CstateC0Time) => self.cstate_c0_time.register("cpu/cstate/c0/time");
            if stats.contains(&CstateC1Time) => self.cstate_c1_time.register("cpu/cstate/c1/time");
            if stats.contains(&CstateC1ETime) => self.cstate_c1e_time.register("cpu/cstate/c1e/time");
            if stats.contains(&CstateC2Time) => self.cstate_c2_time.register("cpu/cstate/c2/time");
            if stats.contains(&CstateC3Time) => self.cstate_c3_time.register("cpu/cstate/c3/time");
            if stats.contains(&CstateC6Time) => self.cstate_c6_time.register("cpu/cstate/c6/time");
            if stats.contains(&CstateC7Time) => self.cstate_c7_time.register("cpu/cstate/c7/time");
            if stats.contains(&CstateC8Time) => self.cstate_c8_time.register("cpu/cstate/c8/time");
            if stats.contains(&Frequency) => self.frequency.register("cpu/frequency");
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumIter,
    EnumString,
    Eq,
    IntoStaticStr,
    PartialEq,
    Hash,
    Serialize,
    AsStaticStr,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum CpuStatistic {
    #[strum(serialize = "cpu/usage/user")]
    UsageUser,
    #[strum(serialize = "cpu/usage/nice")]
    UsageNice,
    #[strum(serialize = "cpu/usage/system")]
    UsageSystem,
    #[strum(serialize = "cpu/usage/idle")]
    UsageIdle,
    #[strum(serialize = "cpu/usage/irq")]
    UsageIrq,
    #[strum(serialize = "cpu/usage/softirq")]
    UsageSoftirq,
    #[strum(serialize = "cpu/usage/steal")]
    UsageSteal,
    #[strum(serialize = "cpu/usage/guest")]
    UsageGuest,
    #[strum(serialize = "cpu/usage/guestnice")]
    UsageGuestNice,
    #[strum(serialize = "cpu/cache/miss")]
    CacheMiss,
    #[strum(serialize = "cpu/cache/access")]
    CacheAccess,
    #[strum(serialize = "cpu/bpu/branch")]
    BpuBranches,
    #[strum(serialize = "cpu/bpu/miss")]
    BpuMiss,
    #[strum(serialize = "cpu/cycles")]
    Cycles,
    #[strum(serialize = "cpu/dtlb/load/miss")]
    DtlbLoadMiss,
    #[strum(serialize = "cpu/dtlb/load/access")]
    DtlbLoadAccess,
    #[strum(serialize = "cpu/dtlb/store/access")]
    DtlbStoreAccess,
    #[strum(serialize = "cpu/dtlb/store/miss")]
    DtlbStoreMiss,
    #[strum(serialize = "cpu/instructions")]
    Instructions,
    #[strum(serialize = "cpu/reference_cycles")]
    ReferenceCycles,
    #[strum(serialize = "cpu/cstate/c0/time")]
    CstateC0Time,
    #[strum(serialize = "cpu/cstate/c1/time")]
    CstateC1Time,
    #[strum(serialize = "cpu/cstate/c1e/time")]
    CstateC1ETime,
    #[strum(serialize = "cpu/cstate/c2/time")]
    CstateC2Time,
    #[strum(serialize = "cpu/cstate/c3/time")]
    CstateC3Time,
    #[strum(serialize = "cpu/cstate/c6/time")]
    CstateC6Time,
    #[strum(serialize = "cpu/cstate/c7/time")]
    CstateC7Time,
    #[strum(serialize = "cpu/cstate/c8/time")]
    CstateC8Time,
    #[strum(serialize = "cpu/frequency")]
    Frequency,
}

impl TryFrom<&str> for CpuStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        CpuStatistic::from_str(s)
    }
}

impl Statistic<AtomicU64, AtomicU32> for CpuStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        match self {
            Self::Frequency => Source::Gauge,
            _ => Source::Counter,
        }
    }
}

impl CpuStatistic {
    #[cfg(feature = "bpf")]
    pub fn event(self) -> Option<Event> {
        match self {
            Self::BpuBranches => Some(Event::Hardware(HardwareEvent::BranchInstructions)),
            Self::BpuMiss => Some(Event::Hardware(HardwareEvent::BranchMisses)),
            Self::CacheAccess => Some(Event::Hardware(HardwareEvent::CacheReferences)),
            Self::CacheMiss => Some(Event::Hardware(HardwareEvent::CacheMisses)),
            Self::Cycles => Some(Event::Hardware(HardwareEvent::CpuCycles)),
            Self::DtlbLoadMiss => Some(Event::HardwareCache(
                CacheId::DTLB,
                CacheOp::Read,
                CacheResult::Miss,
            )),
            Self::DtlbLoadAccess => Some(Event::HardwareCache(
                CacheId::DTLB,
                CacheOp::Read,
                CacheResult::Access,
            )),
            Self::DtlbStoreMiss => Some(Event::HardwareCache(
                CacheId::DTLB,
                CacheOp::Write,
                CacheResult::Miss,
            )),
            Self::DtlbStoreAccess => Some(Event::HardwareCache(
                CacheId::DTLB,
                CacheOp::Write,
                CacheResult::Access,
            )),
            Self::Instructions => Some(Event::Hardware(HardwareEvent::Instructions)),
            Self::ReferenceCycles => Some(Event::Hardware(HardwareEvent::RefCpuCycles)),
            _ => None,
        }
    }

    pub fn table(self) -> Option<&'static str> {
        match self {
            Self::BpuBranches => Some("branch_instructions"),
            Self::BpuMiss => Some("branch_misses"),
            Self::CacheMiss => Some("cache_misses"),
            Self::CacheAccess => Some("cache_references"),
            Self::Cycles => Some("cycles"),
            Self::DtlbLoadMiss => Some("dtlb_load_miss"),
            Self::DtlbLoadAccess => Some("dtlb_load_access"),
            Self::DtlbStoreMiss => Some("dtlb_store_miss"),
            Self::DtlbStoreAccess => Some("dtlb_store_access"),
            Self::Instructions => Some("instructions"),
            Self::ReferenceCycles => Some("reference_cycles"),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ParseCStateError;

impl std::fmt::Display for ParseCStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error parsing cstate")
    }
}

impl std::error::Error for ParseCStateError {
    fn description(&self) -> &str {
        "Error parsing cstate"
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
pub enum CState {
    C0,
    C1,
    C1E,
    C2,
    C3,
    C6,
    C7,
    C8,
}

impl FromStr for CState {
    type Err = ParseCStateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "POLL" | "C0" => Ok(CState::C0),
            "C1" => Ok(CState::C1),
            "C1E" => Ok(CState::C1E),
            "C2" => Ok(CState::C2),
            "C3" => Ok(CState::C3),
            "C6" => Ok(CState::C6),
            "C7" => Ok(CState::C7),
            "C8" => Ok(CState::C8),
            _ => Err(ParseCStateError),
        }
    }
}
