// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde_derive::Deserialize;
use strum::IntoEnumIterator;

use crate::config::SamplerConfig;

use super::stat::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TcpConfig {
    #[serde(default)]
    bpf: bool,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    interval: Option<usize>,
    #[serde(default = "crate::common::default_percentiles")]
    percentiles: Vec<f64>,
    #[serde(default = "default_statistics")]
    statistics: Vec<TcpStatistic>,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            bpf: Default::default(),
            enabled: Default::default(),
            interval: Default::default(),
            percentiles: crate::common::default_percentiles(),
            statistics: default_statistics(),
        }
    }
}

fn default_statistics() -> Vec<TcpStatistic> {
    TcpStatistic::iter().collect()
}

impl SamplerConfig for TcpConfig {
    type Statistic = TcpStatistic;

    fn bpf(&self) -> bool {
        self.bpf
    }

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
        let mut enabled = Vec::new();
        for statistic in self.statistics.iter() {
            if statistic.bpf_table().is_some() {
                if self.bpf() {
                    enabled.push(*statistic);
                }
            } else {
                enabled.push(*statistic);
            }
        }
        enabled
    }
}
