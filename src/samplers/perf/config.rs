// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::SamplerConfig;
use atomics::*;
use metrics::Percentile;
use serde_derive::*;

use super::stat::PerfStatistic;

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

fn default_statistics() -> Vec<PerfStatistic> {
    vec![
        PerfStatistic::ContextSwitches,
        PerfStatistic::CpuMigrations,
        // PerfStatistic::MemoryLoads,
        // PerfStatistic::MemoryLoadMisses,
        // PerfStatistic::MemoryStores,
        // PerfStatistic::MemoryStoreMisses,
        // PerfStatistic::PageFaults,
    ]
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
