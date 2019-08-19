// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::samplers::Statistic;
use perfcnt::linux::HardwareEventType as Hardware;
use perfcnt::linux::PerfCounterBuilderLinux as Builder;
use perfcnt::linux::SoftwareEventType as Software;
use perfcnt::linux::{CacheId, CacheOpId, CacheOpResultId};
use serde_derive::*;

use std::fmt;

#[derive(Clone, Copy, Deserialize, Hash, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum PerfStatistic {
    CacheMisses,
    CacheReferences,
    ContextSwitches,
    CpuBranchInstructions,
    CpuBranchMisses,
    CpuCycles,
    CpuInstructions,
    CpuMigrations,
    CpuRefCycles,
    DtlbLoads,
    DtlbLoadMisses,
    DtlbStores,
    DtlbStoreMisses,
    MemoryLoads,
    MemoryLoadMisses,
    MemoryStores,
    MemoryStoreMisses,
    PageFaults,
    StalledCyclesBackend,
    StalledCyclesFrontend,
}

impl PerfStatistic {
    pub fn builder(self) -> Builder {
        match self {
            PerfStatistic::CacheMisses => Builder::from_hardware_event(Hardware::CacheMisses),
            PerfStatistic::CacheReferences => {
                Builder::from_hardware_event(Hardware::CacheReferences)
            }
            PerfStatistic::ContextSwitches => {
                Builder::from_software_event(Software::ContextSwitches)
            }
            PerfStatistic::CpuBranchInstructions => {
                Builder::from_hardware_event(Hardware::BranchInstructions)
            }
            PerfStatistic::CpuBranchMisses => Builder::from_hardware_event(Hardware::BranchMisses),
            PerfStatistic::CpuCycles => Builder::from_hardware_event(Hardware::CPUCycles),
            PerfStatistic::CpuInstructions => Builder::from_hardware_event(Hardware::Instructions),
            PerfStatistic::CpuMigrations => Builder::from_software_event(Software::CpuMigrations),
            PerfStatistic::CpuRefCycles => Builder::from_hardware_event(Hardware::RefCPUCycles),
            PerfStatistic::DtlbLoads => {
                Builder::from_cache_event(CacheId::DTLB, CacheOpId::Read, CacheOpResultId::Access)
            }
            PerfStatistic::DtlbLoadMisses => {
                Builder::from_cache_event(CacheId::DTLB, CacheOpId::Read, CacheOpResultId::Miss)
            }
            PerfStatistic::DtlbStores => {
                Builder::from_cache_event(CacheId::DTLB, CacheOpId::Write, CacheOpResultId::Access)
            }
            PerfStatistic::DtlbStoreMisses => {
                Builder::from_cache_event(CacheId::DTLB, CacheOpId::Write, CacheOpResultId::Miss)
            }
            PerfStatistic::MemoryLoads => {
                Builder::from_cache_event(CacheId::NODE, CacheOpId::Read, CacheOpResultId::Access)
            }
            PerfStatistic::MemoryLoadMisses => {
                Builder::from_cache_event(CacheId::NODE, CacheOpId::Read, CacheOpResultId::Miss)
            }
            PerfStatistic::MemoryStores => {
                Builder::from_cache_event(CacheId::NODE, CacheOpId::Write, CacheOpResultId::Access)
            }
            PerfStatistic::MemoryStoreMisses => {
                Builder::from_cache_event(CacheId::NODE, CacheOpId::Write, CacheOpResultId::Miss)
            }
            PerfStatistic::PageFaults => Builder::from_software_event(Software::PageFaults),
            PerfStatistic::StalledCyclesBackend => {
                Builder::from_hardware_event(Hardware::StalledCyclesBackend)
            }
            PerfStatistic::StalledCyclesFrontend => {
                Builder::from_hardware_event(Hardware::StalledCyclesFrontend)
            }
        }
    }
}

impl Statistic for PerfStatistic {}

impl fmt::Display for PerfStatistic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PerfStatistic::CacheMisses => write!(f, "perf/cache/misses"),
            PerfStatistic::CacheReferences => write!(f, "perf/cache/references"),
            PerfStatistic::ContextSwitches => write!(f, "perf/system/context_switches"),
            PerfStatistic::CpuBranchInstructions => write!(f, "perf/cpu/branch_instructions"),
            PerfStatistic::CpuBranchMisses => write!(f, "perf/cpu/branch_misses"),
            PerfStatistic::CpuCycles => write!(f, "perf/cpu/cycles"),
            PerfStatistic::CpuInstructions => write!(f, "perf/cpu/instructions"),
            PerfStatistic::CpuMigrations => write!(f, "perf/system/cpu_migrations"),
            PerfStatistic::CpuRefCycles => write!(f, "perf/cpu/reference_cycles"),
            PerfStatistic::DtlbLoads => write!(f, "perf/cache/dtlb/read/references"),
            PerfStatistic::DtlbLoadMisses => write!(f, "perf/cache/dtlb/read/misses"),
            PerfStatistic::DtlbStores => write!(f, "perf/cache/dtlb/write/references"),
            PerfStatistic::DtlbStoreMisses => write!(f, "perf/cache/dtlb/write/misses"),
            PerfStatistic::MemoryLoads => write!(f, "perf/memory/read/references"),
            PerfStatistic::MemoryLoadMisses => write!(f, "perf/memory/read/misses"),
            PerfStatistic::MemoryStores => write!(f, "perf/memory/write/references"),
            PerfStatistic::MemoryStoreMisses => write!(f, "perf/memory/write/misses"),
            PerfStatistic::PageFaults => write!(f, "perf/system/page_faults"),
            PerfStatistic::StalledCyclesBackend => write!(f, "perf/cpu/cycles/stalled/backend"),
            PerfStatistic::StalledCyclesFrontend => write!(f, "perf/cpu/cycles/stalled/frontend"),
        }
    }
}
