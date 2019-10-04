// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;
use crate::samplers::perf::statistics::Statistic;

use atomics::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Perf {
    #[serde(default = "default_enabled")]
    enabled: AtomicBool,
    #[serde(default = "default_interval")]
    interval: AtomicOption<AtomicUsize>,
    #[serde(default = "default_statistics")]
    statistics: Vec<Statistic>,
}

impl Default for Perf {
    fn default() -> Perf {
        Perf {
            enabled: default_enabled(),
            interval: default_interval(),
            statistics: default_statistics(),
        }
    }
}

fn default_enabled() -> AtomicBool {
    AtomicBool::new(false)
}

fn default_interval() -> AtomicOption<AtomicUsize> {
    AtomicOption::none()
}

fn default_statistics() -> Vec<Statistic> {
    vec![
        Statistic::CacheMisses,
        Statistic::CacheReferences,
        Statistic::ContextSwitches,
        Statistic::CpuBranchInstructions,
        Statistic::CpuBranchMisses,
        Statistic::CpuCycles,
        Statistic::CpuInstructions,
        Statistic::CpuMigrations,
        Statistic::CpuRefCycles,
        Statistic::DtlbLoads,
        Statistic::DtlbLoadMisses,
        Statistic::DtlbStores,
        Statistic::DtlbStoreMisses,
        Statistic::MemoryLoads,
        Statistic::MemoryLoadMisses,
        Statistic::MemoryStores,
        Statistic::MemoryStoreMisses,
        Statistic::PageFaults,
        Statistic::StalledCyclesBackend,
        Statistic::StalledCyclesFrontend,
    ]
}

impl SamplerConfig for Perf {
    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    fn interval(&self) -> Option<usize> {
        self.interval.load(Ordering::Relaxed)
    }
}

impl Perf {
    pub fn statistics(&self) -> &[Statistic] {
        &self.statistics
    }
}
