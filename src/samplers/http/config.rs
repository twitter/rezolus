// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde_derive::Deserialize;

use crate::config::SamplerConfig;

use super::stat::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpConfig {
    counters: Vec<String>,
    #[serde(default)]
    enabled: bool,
    gauges: Vec<String>,
    #[serde(default)]
    interval: Option<usize>,
    #[serde(default)]
    passthrough: bool,
    #[serde(default = "crate::common::default_percentiles")]
    percentiles: Vec<f64>,
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
            percentiles: crate::common::default_percentiles(),
            url: None,
        }
    }
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
        self.passthrough
    }
}

impl SamplerConfig for HttpConfig {
    type Statistic = HttpStatistic;

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn interval(&self) -> Option<usize> {
        self.interval
    }

    fn percentiles(&self) -> &[f64] {
        &self.percentiles
    }

    fn statistics(&self) -> Vec<<Self as SamplerConfig>::Statistic> {
        // we don't know the statistics yet, register at runtime instead
        Vec::new()
    }
}
