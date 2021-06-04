// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde_derive::Deserialize;
use std::collections::HashMap;

use crate::config::SamplerConfig;

use super::stat::UsercallStatistic;

pub type LibSearchMap = HashMap<String, Vec<String>>;
pub type LibFileMap = HashMap<String, HashMap<String, Vec<String>>>;

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct UsercallConfig {
    #[serde(default)]
    bpf: bool,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    interval: Option<usize>,
    #[serde(default)]
    percentiles: Vec<f64>,
    #[serde(default)]
    lib_files: LibFileMap,
    #[serde(default)]
    lib_search: LibSearchMap,
}

impl UsercallConfig {
    pub fn lib_files(&self) -> LibFileMap {
        self.lib_files.clone()
    }

    pub fn lib_search(&self) -> LibSearchMap {
        self.lib_search.clone()
    }
}

impl SamplerConfig for UsercallConfig {
    type Statistic = UsercallStatistic;

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
                stats.push(UsercallStatistic::new(lib, func));
            }
        }
        for (lib, funcs) in &self.lib_search {
            // Do not add if this lib is already defined by a lib_file
            if self.lib_files.contains_key(lib) {
                continue;
            }
            for func in funcs.iter() {
                stats.push(UsercallStatistic::new(lib, func));
            }
        }
        stats
    }
}
