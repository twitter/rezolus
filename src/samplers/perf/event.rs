// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use logger::*;
use perfcnt::linux::HardwareEventType as Hardware;
use perfcnt::linux::PerfCounterBuilderLinux as Builder;
use perfcnt::linux::SoftwareEventType as Software;
use perfcnt::linux::{CacheId, CacheOpId, CacheOpResultId};

use std::fmt;

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub enum Event {
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

impl Event {
    pub fn builder(self) -> Builder {
        match self {
            Event::CacheMisses => Builder::from_hardware_event(Hardware::CacheMisses),
            Event::CacheReferences => Builder::from_hardware_event(Hardware::CacheReferences),
            Event::ContextSwitches => Builder::from_software_event(Software::ContextSwitches),
            Event::CpuBranchInstructions => {
                Builder::from_hardware_event(Hardware::BranchInstructions)
            }
            Event::CpuBranchMisses => Builder::from_hardware_event(Hardware::BranchMisses),
            Event::CpuCycles => Builder::from_hardware_event(Hardware::CPUCycles),
            Event::CpuInstructions => Builder::from_hardware_event(Hardware::Instructions),
            Event::CpuMigrations => Builder::from_software_event(Software::CpuMigrations),
            Event::CpuRefCycles => Builder::from_hardware_event(Hardware::RefCPUCycles),
            Event::DtlbLoads => {
                Builder::from_cache_event(CacheId::DTLB, CacheOpId::Read, CacheOpResultId::Access)
            }
            Event::DtlbLoadMisses => {
                Builder::from_cache_event(CacheId::DTLB, CacheOpId::Read, CacheOpResultId::Miss)
            }
            Event::DtlbStores => {
                Builder::from_cache_event(CacheId::DTLB, CacheOpId::Write, CacheOpResultId::Access)
            }
            Event::DtlbStoreMisses => {
                Builder::from_cache_event(CacheId::DTLB, CacheOpId::Write, CacheOpResultId::Miss)
            }
            Event::MemoryLoads => {
                Builder::from_cache_event(CacheId::NODE, CacheOpId::Read, CacheOpResultId::Access)
            }
            Event::MemoryLoadMisses => {
                Builder::from_cache_event(CacheId::NODE, CacheOpId::Read, CacheOpResultId::Miss)
            }
            Event::MemoryStores => {
                Builder::from_cache_event(CacheId::NODE, CacheOpId::Write, CacheOpResultId::Access)
            }
            Event::MemoryStoreMisses => {
                Builder::from_cache_event(CacheId::NODE, CacheOpId::Write, CacheOpResultId::Miss)
            }
            Event::PageFaults => Builder::from_software_event(Software::PageFaults),
            Event::StalledCyclesBackend => {
                Builder::from_hardware_event(Hardware::StalledCyclesBackend)
            }
            Event::StalledCyclesFrontend => {
                Builder::from_hardware_event(Hardware::StalledCyclesFrontend)
            }
        }
    }

    pub fn from_str(event: &str) -> Result<Event, ()> {
        match event {
            "CacheMisses" => Ok(Event::CacheMisses),
            "CacheReferences" => Ok(Event::CacheReferences),
            "ContextSwitches" => Ok(Event::ContextSwitches),
            "CpuBranchInstructions" => Ok(Event::CpuBranchInstructions),
            "CpuBranchMisses" => Ok(Event::CpuBranchMisses),
            "CpuCycles" => Ok(Event::CpuCycles),
            "CpuInstructions" => Ok(Event::CpuInstructions),
            "CpuMigrations" => Ok(Event::CpuMigrations),
            "CpuRefCycles" => Ok(Event::CpuRefCycles),
            "DtlbLoads" => Ok(Event::DtlbLoads),
            "DtlbLoadMisses" => Ok(Event::DtlbLoadMisses),
            "DtlbStores" => Ok(Event::DtlbStores),
            "DtlbStoreMisses" => Ok(Event::DtlbStoreMisses),
            "MemoryLoads" => Ok(Event::MemoryLoads),
            "MemoryLoadMisses" => Ok(Event::MemoryLoadMisses),
            "MemoryStores" => Ok(Event::MemoryStores),
            "MemoryStoreMisses" => Ok(Event::MemoryStoreMisses),
            "PageFaults" => Ok(Event::PageFaults),
            "StalledCyclesBackend" => Ok(Event::StalledCyclesBackend),
            "StalledCyclesFrontend" => Ok(Event::StalledCyclesFrontend),
            _ => {
                debug!("Unknown event: {}", event);
                Err(())
            }
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Event::CacheMisses => write!(f, "cache/misses"),
            Event::CacheReferences => write!(f, "cache/references"),
            Event::ContextSwitches => write!(f, "system/context_switches"),
            Event::CpuBranchInstructions => write!(f, "cpu/branch_instructions"),
            Event::CpuBranchMisses => write!(f, "cpu/branch_misses"),
            Event::CpuCycles => write!(f, "cpu/cycles"),
            Event::CpuInstructions => write!(f, "cpu/instructions"),
            Event::CpuMigrations => write!(f, "system/cpu_migrations"),
            Event::CpuRefCycles => write!(f, "cpu/reference_cycles"),
            Event::DtlbLoads => write!(f, "cache/dtlb/read/references"),
            Event::DtlbLoadMisses => write!(f, "cache/dtlb/read/misses"),
            Event::DtlbStores => write!(f, "cache/dtlb/write/references"),
            Event::DtlbStoreMisses => write!(f, "cache/dtlb/write/misses"),
            Event::MemoryLoads => write!(f, "memory/read/references"),
            Event::MemoryLoadMisses => write!(f, "memory/read/misses"),
            Event::MemoryStores => write!(f, "memory/write/references"),
            Event::MemoryStoreMisses => write!(f, "memory/write/misses"),
            Event::PageFaults => write!(f, "system/page_faults"),
            Event::StalledCyclesBackend => write!(f, "cpu/cycles/stalled/backend"),
            Event::StalledCyclesFrontend => write!(f, "cpu/cycles/stalled/frontend"),
        }
    }
}
