// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;
use crate::samplers::perf::PerfStatistic;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Perf {
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default = "default_statistics")]
    statistics: Vec<PerfStatistic>,
}

impl Default for Perf {
    fn default() -> Perf {
        Perf {
            enabled: default_enabled(),
            statistics: default_statistics(),
        }
    }
}

fn default_enabled() -> bool {
    false
}

fn default_statistics() -> Vec<PerfStatistic> {
    vec![
        PerfStatistic::CacheMisses,
        PerfStatistic::CacheReferences,
        PerfStatistic::ContextSwitches,
        PerfStatistic::CpuBranchInstructions,
        PerfStatistic::CpuBranchMisses,
        PerfStatistic::CpuCycles,
        PerfStatistic::CpuInstructions,
        PerfStatistic::CpuMigrations,
        PerfStatistic::CpuRefCycles,
        PerfStatistic::DtlbLoads,
        PerfStatistic::DtlbLoadMisses,
        PerfStatistic::DtlbStores,
        PerfStatistic::DtlbStoreMisses,
        PerfStatistic::MemoryLoads,
        PerfStatistic::MemoryLoadMisses,
        PerfStatistic::MemoryStores,
        PerfStatistic::MemoryStoreMisses,
        PerfStatistic::PageFaults,
        PerfStatistic::StalledCyclesBackend,
        PerfStatistic::StalledCyclesFrontend,
    ]
}

impl Perf {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn statistics(&self) -> &[PerfStatistic] {
        &self.statistics
    }
}
