// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde_derive::Deserialize;
use strum::IntoEnumIterator;

use crate::config::SamplerConfig;

use super::stat::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RezolusConfig {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    interval: Option<usize>,
    #[serde(default = "crate::common::default_percentiles")]
    percentiles: Vec<f64>,
    #[serde(default = "default_statistics")]
    statistics: Vec<RezolusStatistic>,
}

impl Default for RezolusConfig {
    fn default() -> Self {
        Self {
            enabled: Default::default(),
            interval: Default::default(),
            percentiles: crate::common::default_percentiles(),
            statistics: default_statistics(),
        }
    }
}

fn default_statistics() -> Vec<RezolusStatistic> {
    RezolusStatistic::iter().collect()
}

impl SamplerConfig for RezolusConfig {
    type Statistic = RezolusStatistic;

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
        self.statistics.clone()
    }
}
