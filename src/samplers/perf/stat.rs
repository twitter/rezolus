// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
#[cfg(feature = "perf")]
use perfcnt::linux::HardwareEventType as Hardware;
#[cfg(feature = "perf")]
use perfcnt::linux::PerfCounterBuilderLinux as Builder;
#[cfg(feature = "perf")]
use perfcnt::linux::SoftwareEventType as Software;
#[cfg(feature = "perf")]
use perfcnt::linux::{CacheId, CacheOpId, CacheOpResultId};
use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum PerfStatistic {
    // CacheMisses,
    // CacheReferences,
    ContextSwitches,
    // CpuBranchInstructions,
    // CpuBranchMisses,
    // CpuCycles,
    // CpuInstructions,
    CpuMigrations,
    // CpuRefCycles,
    // DtlbLoads,
    // DtlbLoadMisses,
    // DtlbStores,
    // DtlbStoreMisses,
    // MemoryLoads,
    // MemoryLoadMisses,
    // MemoryStores,
    // MemoryStoreMisses,
    // PageFaults,
    // StalledCyclesBackend,
    // StalledCyclesFrontend,
}

#[cfg(feature = "perf")]
impl PerfStatistic {
    pub fn builder(self) -> Builder {
        match self {
            // Self::CacheMisses => Builder::from_hardware_event(Hardware::CacheMisses),
            // Self::CacheReferences => Builder::from_hardware_event(Hardware::CacheReferences),
            Self::ContextSwitches => Builder::from_software_event(Software::ContextSwitches),
            // Self::CpuBranchInstructions => {
            // Builder::from_hardware_event(Hardware::BranchInstructions)
            // }
            // Self::CpuBranchMisses => Builder::from_hardware_event(Hardware::BranchMisses),
            // Self::CpuCycles => Builder::from_hardware_event(Hardware::CPUCycles),
            // Self::CpuInstructions => Builder::from_hardware_event(Hardware::Instructions),
            Self::CpuMigrations => Builder::from_software_event(Software::CpuMigrations),
            // Self::CpuRefCycles => Builder::from_hardware_event(Hardware::RefCPUCycles),
            // Self::DtlbLoads => {
            // Builder::from_cache_event(CacheId::DTLB, CacheOpId::Read, CacheOpResultId::Access)
            // }
            // Self::DtlbLoadMisses => {
            // Builder::from_cache_event(CacheId::DTLB, CacheOpId::Read, CacheOpResultId::Miss)
            // }
            // Self::DtlbStores => {
            // Builder::from_cache_event(CacheId::DTLB, CacheOpId::Write, CacheOpResultId::Access)
            // }
            // Self::DtlbStoreMisses => {
            // Builder::from_cache_event(CacheId::DTLB, CacheOpId::Write, CacheOpResultId::Miss)
            // }
            // Self::MemoryLoads => {
            //     Builder::from_cache_event(CacheId::NODE, CacheOpId::Read, CacheOpResultId::Access)
            // }
            // Self::MemoryLoadMisses => {
            //     Builder::from_cache_event(CacheId::NODE, CacheOpId::Read, CacheOpResultId::Miss)
            // }
            // Self::MemoryStores => {
            //     Builder::from_cache_event(CacheId::NODE, CacheOpId::Write, CacheOpResultId::Access)
            // }
            // Self::MemoryStoreMisses => {
            //     Builder::from_cache_event(CacheId::NODE, CacheOpId::Write, CacheOpResultId::Miss)
            // }
            // Self::PageFaults => Builder::from_software_event(Software::PageFaults),
            // Self::StalledCyclesBackend => {
            //     Builder::from_hardware_event(Hardware::StalledCyclesBackend)
            // }
            // Self::StalledCyclesFrontend => {
            //     Builder::from_hardware_event(Hardware::StalledCyclesFrontend)
            // }
        }
    }
}

impl Statistic for PerfStatistic {
    fn name(&self) -> &str {
        match self {
            // Self::CacheMisses => "cpu/cache/misses",
            // Self::CacheReferences => "cpu/cache/references",
            // Self::CpuBranchInstructions => "cpu/branch_prediction/instructions",
            // Self::CpuBranchMisses => "cpu/branch_prediction/misses",
            // Self::CpuCycles => "cpu/cycles",
            // Self::CpuInstructions => "cpu/instructions",
            Self::CpuMigrations => "scheduler/cpu_migrations",
            Self::ContextSwitches => "scheduler/context_switches",
            // Self::CpuRefCycles => "cpu/reference_cycles",
            // Self::DtlbLoads => "cpu/dtlb/load",
            // Self::DtlbLoadMisses => "cpu/dtlb/load/misses",
            // Self::DtlbStores => "cpu/dtlb/store",
            // Self::DtlbStoreMisses => "cpu/dtlb/store/misses",
            // Self::MemoryLoads => "memory/load",
            // Self::MemoryStores => "memory/store",
            // Self::MemoryStoreMisses => "memory/store/misses",
            // Self::MemoryLoadMisses => "memory/load/misses",
            // Self::PageFaults => "cpu/page_fault",
            // Self::StalledCyclesFrontend => "cpu/stalled_cycles/frontend",
            // Self::StalledCyclesBackend => "cpu/stalled_cycles/backend",
        }
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Counter
    }
}
