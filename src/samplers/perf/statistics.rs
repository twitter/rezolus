// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use perfcnt::linux::HardwareEventType as Hardware;
use perfcnt::linux::PerfCounterBuilderLinux as Builder;
use perfcnt::linux::SoftwareEventType as Software;
use perfcnt::linux::{CacheId, CacheOpId, CacheOpResultId};
use serde_derive::*;

use std::fmt;

#[derive(Clone, Copy, Deserialize, Hash, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Statistic {
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

impl Statistic {
    pub fn builder(self) -> Builder {
        match self {
            Self::CacheMisses => Builder::from_hardware_event(Hardware::CacheMisses),
            Self::CacheReferences => {
                Builder::from_hardware_event(Hardware::CacheReferences)
            }
            Self::ContextSwitches => {
                Builder::from_software_event(Software::ContextSwitches)
            }
            Self::CpuBranchInstructions => {
                Builder::from_hardware_event(Hardware::BranchInstructions)
            }
            Self::CpuBranchMisses => Builder::from_hardware_event(Hardware::BranchMisses),
            Self::CpuCycles => Builder::from_hardware_event(Hardware::CPUCycles),
            Self::CpuInstructions => Builder::from_hardware_event(Hardware::Instructions),
            Self::CpuMigrations => Builder::from_software_event(Software::CpuMigrations),
            Self::CpuRefCycles => Builder::from_hardware_event(Hardware::RefCPUCycles),
            Self::DtlbLoads => {
                Builder::from_cache_event(CacheId::DTLB, CacheOpId::Read, CacheOpResultId::Access)
            }
            Self::DtlbLoadMisses => {
                Builder::from_cache_event(CacheId::DTLB, CacheOpId::Read, CacheOpResultId::Miss)
            }
            Self::DtlbStores => {
                Builder::from_cache_event(CacheId::DTLB, CacheOpId::Write, CacheOpResultId::Access)
            }
            Self::DtlbStoreMisses => {
                Builder::from_cache_event(CacheId::DTLB, CacheOpId::Write, CacheOpResultId::Miss)
            }
            Self::MemoryLoads => {
                Builder::from_cache_event(CacheId::NODE, CacheOpId::Read, CacheOpResultId::Access)
            }
            Self::MemoryLoadMisses => {
                Builder::from_cache_event(CacheId::NODE, CacheOpId::Read, CacheOpResultId::Miss)
            }
            Self::MemoryStores => {
                Builder::from_cache_event(CacheId::NODE, CacheOpId::Write, CacheOpResultId::Access)
            }
            Self::MemoryStoreMisses => {
                Builder::from_cache_event(CacheId::NODE, CacheOpId::Write, CacheOpResultId::Miss)
            }
            Self::PageFaults => Builder::from_software_event(Software::PageFaults),
            Self::StalledCyclesBackend => {
                Builder::from_hardware_event(Hardware::StalledCyclesBackend)
            }
            Self::StalledCyclesFrontend => {
                Builder::from_hardware_event(Hardware::StalledCyclesFrontend)
            }
        }
    }
}

impl fmt::Display for Statistic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::CacheMisses => write!(f, "perf/cache/misses"),
            Self::CacheReferences => write!(f, "perf/cache/references"),
            Self::ContextSwitches => write!(f, "perf/system/context_switches"),
            Self::CpuBranchInstructions => write!(f, "perf/cpu/branch_instructions"),
            Self::CpuBranchMisses => write!(f, "perf/cpu/branch_misses"),
            Self::CpuCycles => write!(f, "perf/cpu/cycles"),
            Self::CpuInstructions => write!(f, "perf/cpu/instructions"),
            Self::CpuMigrations => write!(f, "perf/system/cpu_migrations"),
            Self::CpuRefCycles => write!(f, "perf/cpu/reference_cycles"),
            Self::DtlbLoads => write!(f, "perf/cache/dtlb/read/references"),
            Self::DtlbLoadMisses => write!(f, "perf/cache/dtlb/read/misses"),
            Self::DtlbStores => write!(f, "perf/cache/dtlb/write/references"),
            Self::DtlbStoreMisses => write!(f, "perf/cache/dtlb/write/misses"),
            Self::MemoryLoads => write!(f, "perf/memory/read/references"),
            Self::MemoryLoadMisses => write!(f, "perf/memory/read/misses"),
            Self::MemoryStores => write!(f, "perf/memory/write/references"),
            Self::MemoryStoreMisses => write!(f, "perf/memory/write/misses"),
            Self::PageFaults => write!(f, "perf/system/page_faults"),
            Self::StalledCyclesBackend => write!(f, "perf/cpu/cycles/stalled/backend"),
            Self::StalledCyclesFrontend => write!(f, "perf/cpu/cycles/stalled/frontend"),
        }
    }
}