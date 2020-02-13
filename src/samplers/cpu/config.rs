// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::*;
use serde_derive::Deserialize;

use crate::config::SamplerConfig;

use super::stat::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CpuConfig {
    #[serde(default)]
    enabled: AtomicBool,
    #[serde(default)]
    interval: Option<AtomicUsize>,
    #[serde(default = "default_percentiles")]
    percentiles: Vec<Percentile>,
    #[serde(default)]
    perf_events: AtomicBool,
    #[serde(default = "default_statistics")]
    statistics: Vec<CpuStatistic>,
}

impl Default for CpuConfig {
    fn default() -> Self {
        Self {
            enabled: Default::default(),
            interval: Default::default(),
            percentiles: default_percentiles(),
            perf_events: Default::default(),
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

fn default_statistics() -> Vec<CpuStatistic> {
    vec![
        CpuStatistic::UsageUser,
        CpuStatistic::UsageSystem,
        CpuStatistic::UsageIdle,
    ]
}

impl SamplerConfig for CpuConfig {
    type Statistic = CpuStatistic;
    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    fn interval(&self) -> Option<usize> {
        self.interval.as_ref().map(|v| v.load(Ordering::Relaxed))
    }

    fn percentiles(&self) -> &[Percentile] {
        &self.percentiles
    }

    fn perf_events(&self) -> bool {
        self.perf_events.load(Ordering::Relaxed)
    }

    fn statistics(&self) -> &[<Self as SamplerConfig>::Statistic] {
        &self.statistics
    }
}
