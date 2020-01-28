// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::*;
use serde_derive::Deserialize;

use crate::config::SamplerConfig;

use super::stat::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XfsConfig {
    #[serde(default)]
    ebpf: AtomicBool,
    #[serde(default)]
    enabled: AtomicBool,
    #[serde(default)]
    interval: Option<AtomicUsize>,
    #[serde(default = "default_percentiles")]
    percentiles: Vec<Percentile>,
    #[serde(default = "default_statistics")]
    statistics: Vec<XfsStatistic>,
}

impl Default for XfsConfig {
    fn default() -> Self {
        Self {
            ebpf: Default::default(),
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

fn default_statistics() -> Vec<XfsStatistic> {
    vec![
        XfsStatistic::FsyncLatency,
        XfsStatistic::OpenLatency,
        XfsStatistic::ReadLatency,
        XfsStatistic::WriteLatency,
    ]
}

impl SamplerConfig for XfsConfig {
    type Statistic = XfsStatistic;

    fn ebpf(&self) -> bool {
        self.ebpf.load(Ordering::Relaxed)
    }

    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    fn interval(&self) -> Option<usize> {
        self.interval.as_ref().map(|v| v.load(Ordering::Relaxed))
    }

    fn percentiles(&self) -> &[Percentile] {
        &self.percentiles
    }

    fn statistics(&self) -> &[<Self as SamplerConfig>::Statistic] {
        &self.statistics
    }
}
