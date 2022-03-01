// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::io::{BufRead, BufReader};

use serde_derive::Deserialize;
use strum::IntoEnumIterator;

use crate::config::SamplerConfig;

use super::stat::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfig {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    interval: Option<usize>,
    #[serde(default = "crate::common::default_percentiles")]
    percentiles: Vec<f64>,
    #[serde(default = "default_statistics")]
    statistics: Vec<ProcessStatistic>,
    #[serde(default)]
    pid_file: Option<String>,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            enabled: Default::default(),
            interval: Default::default(),
            percentiles: crate::common::default_percentiles(),
            statistics: default_statistics(),
            pid_file: Default::default(),
        }
    }
}

fn default_statistics() -> Vec<ProcessStatistic> {
    ProcessStatistic::iter().collect()
}

impl SamplerConfig for ProcessConfig {
    type Statistic = ProcessStatistic;

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

impl ProcessConfig {
    pub fn pid(&self) -> Option<u32> {
        if let Some(filename) = &self.pid_file {
            if let Ok(file) = std::fs::File::open(filename) {
                let mut buffer = BufReader::new(file);
                let mut line = String::new();
                if buffer.read_line(&mut line).is_ok() {
                    let line = line.trim();
                    if let Ok(pid) = line.parse::<u32>() {
                        return Some(pid);
                    } else {
                        debug!("PID file did not parse: {}", line);
                    }
                } else {
                    debug!("failed to read line from PID file");
                }
            } else {
                debug!("failed to open PID file");
            }
        } else {
            debug!("no PID file provided");
        }
        None
    }
}
