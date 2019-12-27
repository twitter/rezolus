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
        MemoryStatistic::MemTotal,
        MemoryStatistic::MemFree,
        MemoryStatistic::MemAvailable,
        MemoryStatistic::Buffers,
        MemoryStatistic::Cached,
        MemoryStatistic::SwapCached,
        MemoryStatistic::Active,
        MemoryStatistic::Inactive,
        MemoryStatistic::ActiveAnon,
        MemoryStatistic::InactiveAnon,
        MemoryStatistic::ActiveFile,
        MemoryStatistic::InactiveFile,
        MemoryStatistic::Unevictable,
        MemoryStatistic::Mlocked,
        MemoryStatistic::SwapTotal,
        MemoryStatistic::SwapFree,
        MemoryStatistic::Dirty,
        MemoryStatistic::Writeback,
        MemoryStatistic::AnonPages,
        MemoryStatistic::Mapped,
        MemoryStatistic::Shmem,
        MemoryStatistic::Slab,
        MemoryStatistic::SReclaimable,
        MemoryStatistic::SUnreclaim,
        MemoryStatistic::KernelStack,
        MemoryStatistic::PageTables,
        MemoryStatistic::NFSUnstable,
        MemoryStatistic::Bounce,
        MemoryStatistic::WritebackTmp,
        MemoryStatistic::CommitLimit,
        MemoryStatistic::CommittedAS,
        MemoryStatistic::VmallocTotal,
        MemoryStatistic::VmallocUsed,
        MemoryStatistic::VmallocChunk,
        MemoryStatistic::Percpu,
        MemoryStatistic::HardwareCorrupted,
        MemoryStatistic::AnonHugePages,
        MemoryStatistic::ShmemHugePages,
        MemoryStatistic::ShmemPmdMapped,
        MemoryStatistic::HugePagesTotal,
        MemoryStatistic::HugePagesFree,
        MemoryStatistic::HugePagesRsvd,
        MemoryStatistic::HugePagesSurp,
        MemoryStatistic::Hugepagesize,
        MemoryStatistic::Hugetlb,
        MemoryStatistic::DirectMap4k,
        MemoryStatistic::DirectMap2M,
        MemoryStatistic::DirectMap1G,
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
