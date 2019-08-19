// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::*;
use crate::config::Config;
use crate::samplers::Sampler;
use crate::stats::{record_counter, register_counter};
use failure::Error;

use logger::*;
use metrics::*;
use serde_derive::*;
use time;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const SOFTNET_STAT: &str = "/proc/net/softnet_stat";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum Statistic {
    Processed,
    Dropped,
    TimeSqueezed,
    CpuCollision,
    ReceivedRps,
    FlowLimitCount,
}

impl std::fmt::Display for Statistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Statistic::Processed => write!(f, "softnet/processed"),
            Statistic::Dropped => write!(f, "softnet/dropped"),
            Statistic::TimeSqueezed => write!(f, "softnet/time_squeezed"),
            Statistic::CpuCollision => write!(f, "softnet/cpu_collision"),
            Statistic::ReceivedRps => write!(f, "softnet/received_rps"),
            Statistic::FlowLimitCount => write!(f, "softnet/flow_limit_count"),
        }
    }
}

impl Statistic {
    fn field_number(&self) -> usize {
        match self {
            Statistic::Processed => 0,
            Statistic::Dropped => 1,
            Statistic::TimeSqueezed => 2,
            Statistic::CpuCollision => 3,
            Statistic::ReceivedRps => 4,
            Statistic::FlowLimitCount => 5,
        }
    }
}

pub struct Softnet<'a> {
    config: &'a Config,
    initialized: bool,
    recorder: &'a Recorder<AtomicU32>,
}

pub fn read_softnet_stat<P: AsRef<Path>>(path: P) -> HashMap<Statistic, u64> {
    let mut result = HashMap::new();
    let file = File::open(path)
        .map_err(|e| debug!("could not read softnet_stat: {}", e))
        .expect("failed to open file");

    let file = BufReader::new(file);
    for line in file.lines() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() != 11 {
            continue;
        }
        for statistic in &[
            Statistic::Processed,
            Statistic::Dropped,
            Statistic::TimeSqueezed,
            Statistic::CpuCollision,
            Statistic::ReceivedRps,
            Statistic::FlowLimitCount,
        ] {
            if !result.contains_key(statistic) {
                result.insert(*statistic, 0);
            }
            let current = result.get_mut(statistic).unwrap();
            *current += u64::from_str_radix(tokens[statistic.field_number()], 16).unwrap_or(0);
        }
    }

    result
}

impl<'a> Sampler<'a> for Softnet<'a> {
    fn new(
        config: &'a Config,
        recorder: &'a Recorder<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
        if config.softnet().enabled() {
            Ok(Some(Box::new(Self {
                config,
                initialized: false,
                recorder,
            })))
        } else {
            Ok(None)
        }
    }

    fn name(&self) -> String {
        "softnet".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        trace!("sample {}", self.name());
        let time = time::precise_time_ns();
        let data = read_softnet_stat(SOFTNET_STAT);
        if !self.initialized {
            self.register();
        }
        for (statistic, value) in data {
            record_counter(self.recorder, statistic, time, value);
        }
        Ok(())
    }

    fn register(&mut self) {
        trace!("register {}", self.name());
        if !self.initialized {
            let data = read_softnet_stat(SOFTNET_STAT);
            for statistic in data.keys() {
                register_counter(
                    self.recorder,
                    statistic.to_string(),
                    TRILLION,
                    3,
                    self.config.general().window(),
                    PERCENTILES,
                );
            }
            self.initialized = true;
        }
    }

    fn deregister(&mut self) {
        trace!("deregister {}", self.name());
        if self.initialized {
            let data = read_softnet_stat(SOFTNET_STAT);
            for statistic in data.keys() {
                self.recorder.delete_channel(statistic.to_string());
            }
            self.initialized = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_softnet_stat() {
        let data = read_softnet_stat(format!("tests/data{}", SOFTNET_STAT));
        assert_eq!(data.get(&Statistic::Processed), Some(&18035263));
        assert_eq!(data.get(&Statistic::Dropped), Some(&0));
        assert_eq!(data.get(&Statistic::TimeSqueezed), Some(&135098));
    }
}
