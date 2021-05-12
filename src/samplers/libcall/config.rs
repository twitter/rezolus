use serde_derive::Deserialize;
use std::collections::HashMap;

use crate::config::SamplerConfig;

use super::stat::LibCallStatistic;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibCallConfig {
    #[serde(default)]
    bpf: bool,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    interval: Option<usize>,
    #[serde(default = "crate::common::default_percentiles")]
    percentiles: Vec<f64>,
    #[serde(default)]
    probe_funcs: HashMap<String, Vec<String>>,
}

impl Default for LibCallConfig {
    fn default() -> Self {
        Self {
            bpf: Default::default(),
            enabled: Default::default(),
            interval: Default::default(),
            percentiles: crate::common::default_percentiles(),
            probe_funcs: HashMap::new(),
        }
    }
}

impl SamplerConfig for LibCallConfig {
    type Statistic = LibCallStatistic;

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
        let mut stats = Vec::new();
        for (lib, funcs) in &self.probe_funcs {
            for func in funcs.iter() {
                stats.push(LibCallStatistic::new(lib, func));
            }
        }
        stats
    }
}
