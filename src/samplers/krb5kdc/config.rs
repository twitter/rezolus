// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde_derive::Deserialize;
use strum::IntoEnumIterator;

use crate::config::SamplerConfig;

use super::stat::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Krb5kdcConfig {
    #[serde(default)]
    bpf: bool,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    interval: Option<usize>,
    #[serde(default)]
    percentiles: Vec<f64>,
    #[serde(default = "default_statistics")]
    statistics: Vec<Krb5kdcStatistic>,
    #[serde(default)]
    path: String,
}

impl Default for Krb5kdcConfig {
    fn default() -> Self {
        Self {
            bpf: Default::default(),
            enabled: Default::default(),
            interval: Default::default(),
            percentiles: crate::common::default_percentiles(),
            statistics: default_statistics(),
            path: Default::default(),
        }
    }
}

impl Krb5kdcConfig {
    pub fn path(&self) -> String {
        self.path.clone()
    }
}

fn default_statistics() -> Vec<Krb5kdcStatistic> {
    Krb5kdcStatistic::iter().collect()
}

impl SamplerConfig for Krb5kdcConfig {
    type Statistic = Krb5kdcStatistic;

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
            enabled.push(*statistic);
        }
        enabled
    }
}
