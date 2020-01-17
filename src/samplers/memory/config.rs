// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::stat::*;
use crate::config::SamplerConfig;
use metrics::Percentile;

use atomics::*;
use serde_derive::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryConfig {
    #[serde(default)]
    enabled: AtomicBool,
    #[serde(default)]
    interval: AtomicOption<AtomicUsize>,
    #[serde(default = "default_percentiles")]
    percentiles: Vec<Percentile>,
    #[serde(default = "default_statistics")]
    statistics: Vec<MemoryStatistic>,
}

impl Default for MemoryConfig {
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

fn default_statistics() -> Vec<MemoryStatistic> {
    vec![
        MemoryStatistic::Total,
        MemoryStatistic::Free,
        MemoryStatistic::Available,
        MemoryStatistic::Buffers,
        MemoryStatistic::Cached,
        MemoryStatistic::Active,
        MemoryStatistic::Inactive,
        MemoryStatistic::Unevictable,
        MemoryStatistic::HardwareCorrupted,
        MemoryStatistic::AnonHugePages,
        MemoryStatistic::HugePagesTotal,
        MemoryStatistic::HugePagesFree,
        MemoryStatistic::HugePagesRsvd,
        MemoryStatistic::HugePagesSurp,
        MemoryStatistic::Hugepagesize,
        MemoryStatistic::Hugetlb,
    ]
}

impl SamplerConfig for MemoryConfig {
    type Statistic = MemoryStatistic;
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
