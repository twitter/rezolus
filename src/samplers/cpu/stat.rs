// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "perf")]
pub use perfcnt::linux::*;

use core::convert::TryFrom;
use core::str::FromStr;
use metrics::Statistic;
use serde_derive::*;

use strum::ParseError;
use strum_macros::*;

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq, Hash, Serialize,
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
    #[strum(serialize = "cpu/stalled_cycles/backend")]
    StalledCyclesBackend,
    #[strum(serialize = "cpu/stalled_cycles/frontend")]
    StalledCyclesFrontend,
}

impl TryFrom<&str> for CpuStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        CpuStatistic::from_str(s)
    }
}

impl Statistic for CpuStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Counter
    }
}

impl CpuStatistic {
    #[cfg(feature = "perf")]
    pub fn perf_counter_builder(&self) -> Option<PerfCounterBuilderLinux> {
        match self {
            Self::BpuBranches => Some(PerfCounterBuilderLinux::from_hardware_event(
                HardwareEventType::BranchInstructions,
            )),
            Self::BpuMiss => Some(PerfCounterBuilderLinux::from_hardware_event(
                HardwareEventType::BranchMisses,
            )),
            Self::CacheMiss => Some(PerfCounterBuilderLinux::from_hardware_event(
                HardwareEventType::CacheMisses,
            )),
            Self::CacheAccess => Some(PerfCounterBuilderLinux::from_hardware_event(
                HardwareEventType::CacheReferences,
            )),
            Self::Cycles => Some(PerfCounterBuilderLinux::from_hardware_event(
                HardwareEventType::CPUCycles,
            )),
            Self::DtlbLoadMiss => Some(PerfCounterBuilderLinux::from_cache_event(
                CacheId::DTLB,
                CacheOpId::Read,
                CacheOpResultId::Miss,
            )),
            Self::DtlbLoadAccess => Some(PerfCounterBuilderLinux::from_cache_event(
                CacheId::DTLB,
                CacheOpId::Read,
                CacheOpResultId::Access,
            )),
            Self::DtlbStoreMiss => Some(PerfCounterBuilderLinux::from_cache_event(
                CacheId::DTLB,
                CacheOpId::Write,
                CacheOpResultId::Miss,
            )),
            Self::DtlbStoreAccess => Some(PerfCounterBuilderLinux::from_cache_event(
                CacheId::DTLB,
                CacheOpId::Write,
                CacheOpResultId::Access,
            )),
            Self::Instructions => Some(PerfCounterBuilderLinux::from_hardware_event(
                HardwareEventType::Instructions,
            )),
            Self::ReferenceCycles => Some(PerfCounterBuilderLinux::from_hardware_event(
                HardwareEventType::RefCPUCycles,
            )),
            Self::StalledCyclesBackend => Some(PerfCounterBuilderLinux::from_hardware_event(
                HardwareEventType::StalledCyclesBackend,
            )),
            Self::StalledCyclesFrontend => Some(PerfCounterBuilderLinux::from_hardware_event(
                HardwareEventType::StalledCyclesFrontend,
            )),
            _ => None,
        }
    }
}
