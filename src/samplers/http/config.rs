// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::sync::atomic::AtomicBool;
use super::stat::*;
use crate::config::SamplerConfig;
use rustcommon_metrics::Percentile;

use rustcommon_atomics::*;
use serde_derive::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpConfig {
    counters: Vec<String>,
    #[serde(default)]
    enabled: AtomicBool,
    gauges: Vec<String>,
    #[serde(default)]
    interval: Option<AtomicUsize>,
    #[serde(default)]
    passthrough: AtomicBool,
    #[serde(default = "default_percentiles")]
    percentiles: Vec<Percentile>,
    url: Option<String>,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            counters: Vec::new(),
            enabled: Default::default(),
            gauges: Vec::new(),
            interval: Default::default(),
            passthrough: Default::default(),
            percentiles: default_percentiles(),
            url: None,
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

impl HttpConfig {
    /// The URL to query metrics from
    pub fn url(&self) -> Option<String> {
        self.url.clone()
    }

    /// A list of metric names that should be processed as gauges with
    /// percentiles
    pub fn gauges(&self) -> &[String] {
        &self.gauges
    }

    /// A list of metric names that should be processed as counters with
    /// percentiles
    pub fn counters(&self) -> &[String] {
        &self.counters
    }

    /// Whether unlisted metrics should be passed through to the output, which
    /// internally treats them as gauges without percentiles
    pub fn passthrough(&self) -> bool {
        self.passthrough.load(Ordering::Relaxed)
    }
}

impl SamplerConfig for HttpConfig {
    type Statistic = HttpStatistic;

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
        &[]
    }
}
