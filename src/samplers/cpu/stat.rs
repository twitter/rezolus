// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use rustcommon_metrics::{Source, Statistic};
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

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

    fn source(&self) -> Source {
        Source::Counter
    }
}

impl CpuStatistic {
    #[cfg(feature = "bpf")]
    pub fn bpf_config(self) -> Option<(&'static str, bcc::perf::Event)> {
        use bcc::perf::{Event, HardwareEvent, CacheId, CacheOp, CacheResult};
        match self {
            Self::BpuBranches => Some(("bpu_branch", Event::Hardware(HardwareEvent::BranchInstructions))),
            Self::BpuMiss => Some(("bpu_miss", Event::Hardware(HardwareEvent::BranchMisses))),  
            Self::CacheAccess => Some(("cache_access", Event::Hardware(HardwareEvent::CacheReferences))),
            Self::CacheMiss => Some(("cache_miss", Event::Hardware(HardwareEvent::CacheMisses))),
            Self::Cycles => Some(("cycles", Event::Hardware(HardwareEvent::CpuCycles))),
            Self::DtlbLoadAccess => Some(("dtlb_load_access", Event::HardwareCache(CacheId::DTLB, CacheOp::Read, CacheResult::Access))),
            Self::DtlbLoadMiss => Some(("dtlb_load_miss", Event::HardwareCache(CacheId::DTLB, CacheOp::Read, CacheResult::Miss))),
            Self::DtlbStoreAccess => Some(("dtlb_store_access", Event::HardwareCache(CacheId::DTLB, CacheOp::Write, CacheResult::Access))),
            Self::DtlbStoreMiss => Some(("dtlb_store_miss", Event::HardwareCache(CacheId::DTLB, CacheOp::Write, CacheResult::Miss))),
            Self::Instructions => Some(("instructions", Event::Hardware(HardwareEvent::Instructions))),
            Self::ReferenceCycles => Some(("ref_cycles", Event::Hardware(HardwareEvent::RefCpuCycles))),
            Self::StalledCyclesBackend => Some(("stalled_backend", Event::Hardware(HardwareEvent::StalledCyclesBackend))),
            Self::StalledCyclesFrontend => Some(("stalled_frontend", Event::Hardware(HardwareEvent::StalledCyclesFrontend))),
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
