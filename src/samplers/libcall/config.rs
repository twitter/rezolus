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
    #[serde(default)]
    percentiles: Vec<f64>,
    #[serde(default)]
    lib_files: HashMap<String, HashMap<String, Vec<String>>>,
    #[serde(default)]
    lib_search: HashMap<String, Vec<String>>,
}

impl Default for LibCallConfig {
    fn default() -> Self {
        Self {
            bpf: Default::default(),
            enabled: Default::default(),
            interval: Default::default(),
            percentiles: vec![],
            lib_search: HashMap::new(),
            lib_files: HashMap::new(),
        }
    }
}

impl LibCallConfig {
    pub fn lib_files(&self) -> HashMap<String, HashMap<String, Vec<String>>> {
        self.lib_files.clone()
    }

    pub fn lib_search(&self) -> HashMap<String, Vec<String>> {
        self.lib_search.clone()
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
        for (lib, func_map) in &self.lib_files {
            for func in func_map.values().flatten() {
                stats.push(LibCallStatistic::new(lib, func));
            }
        }
        for (lib, funcs) in &self.lib_search {
            // Do not add if this lib is already defined by a lib_file
            if self.lib_files.contains_key(lib) {
                continue;
            }
            for func in funcs.iter() {
                stats.push(LibCallStatistic::new(lib, func));
            }
        }
        stats
    }
}
