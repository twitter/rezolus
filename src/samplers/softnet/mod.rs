// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::*;
use crate::config::Config;
use crate::sampler::{Sampler, SamplerError};
use crate::stats::{record_counter, register_counter};

use logger::*;
use metrics::*;
use time;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const SOFTNET_STAT: &str = "/proc/net/softnet_stat";

pub struct Softnet {
    initialized: bool,
}

impl Softnet {
    pub fn new(_config: &Config) -> Self {
        Self { initialized: false }
    }
}

pub fn read_softnet_stat<P: AsRef<Path>>(path: P) -> HashMap<String, u64> {
    let mut result = HashMap::new();
    let file = File::open(path)
        .map_err(|e| debug!("could not read softnet_stat: {}", e))
        .expect("failed to open file");
    let mut dropped = 0;
    let mut processed = 0;
    let mut time_squeezed = 0;

    let file = BufReader::new(file);
    for line in file.lines() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() != 11 {
            continue;
        }
        processed += u64::from_str_radix(tokens[0], 16).unwrap_or(0);
        dropped += u64::from_str_radix(tokens[1], 16).unwrap_or(0);
        time_squeezed += u64::from_str_radix(tokens[2], 16).unwrap_or(0);
    }
    result.insert("processed".to_string(), processed);
    result.insert("dropped".to_string(), dropped);
    result.insert("time_squeezed".to_string(), time_squeezed);
    result
}

impl Sampler for Softnet {
    fn name(&self) -> String {
        "softnet".to_string()
    }

    fn sample(&mut self, recorder: &Recorder<u32>, config: &Config) -> Result<(), SamplerError> {
        trace!("sample {}", self.name());
        let time = time::precise_time_ns();
        let data = read_softnet_stat(SOFTNET_STAT);
        if !self.initialized {
            self.register(recorder, config);
        }
        for (label, value) in data {
            record_counter(recorder, format!("softnet/{}", label), time, value);
        }
        Ok(())
    }

    fn register(&mut self, recorder: &Recorder<u32>, config: &Config) {
        trace!("register {}", self.name());
        if !self.initialized {
            let data = read_softnet_stat(SOFTNET_STAT);
            for label in data.keys() {
                register_counter(
                    recorder,
                    format!("softnet/{}", label),
                    TRILLION,
                    3,
                    config.general().interval(),
                    PERCENTILES,
                );
            }
            self.initialized = true;
        }
    }

    fn deregister(&mut self, recorder: &Recorder<u32>, _config: &Config) {
        trace!("deregister {}", self.name());
        if self.initialized {
            let data = read_softnet_stat(SOFTNET_STAT);
            for label in data.keys() {
                recorder.delete_channel(format!("softnet/{}", label));
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
        assert_eq!(data.get("processed"), Some(&18035263));
        assert_eq!(data.get("dropped"), Some(&0));
        assert_eq!(data.get("time_squeezed"), Some(&135098));
    }
}
