// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::SamplerConfig;
use atomics::*;
use metrics::Percentile;
use serde_derive::*;

#[cfg(feature = "perf")]
use super::stat::PerfStatistic;

#[derive(Clone, Copy, Deserialize, Hash, Debug, PartialEq, Eq)]
#[cfg(not(feature = "perf"))]
pub enum PerfStatistic {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerfConfig {
    #[serde(default)]
    enabled: AtomicBool,
    #[serde(default)]
    interval: AtomicOption<AtomicUsize>,
    #[serde(default = "default_percentiles")]
    percentiles: Vec<Percentile>,
    #[serde(default = "default_statistics")]
    statistics: Vec<PerfStatistic>,
}

impl Default for PerfConfig {
    fn default() -> Self {
        Self {
            enabled: Default::default(),
            interval: Default::default(),
            percentiles: default_percentiles(),
            statistics: default_statistics(),
        }
    }
}

fn default_percentiles() -> Vec<Percentile> {
    vec![
        Percentile::p1,
        Percentile::p10,
        Percentile::p50,
        Percentile::p90,
        Percentile::p99,
    ]
}

#[cfg(feature = "perf")]
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

#[cfg(not(feature = "perf"))]
fn default_statistics() -> Vec<PerfStatistic> {
    vec![]
}

impl SamplerConfig for PerfConfig {
    type Statistic = PerfStatistic;
    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    fn interval(&self) -> Option<usize> {
        self.interval.load(Ordering::Relaxed)
    }

    fn percentiles(&self) -> &[Percentile] {
        &self.percentiles
    }

    fn statistics(&self) -> &[<Self as SamplerConfig>::Statistic] {
        &self.statistics
    }
}
