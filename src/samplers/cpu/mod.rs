// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::*;
use crate::config::Config;
use crate::sampler::{Sampler, SamplerError};
use crate::stats::record_counter;
use crate::stats::register_counter;

use logger::*;
use metrics::*;
use time;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

const PROC_STAT: &str = "/proc/stat";

// reported percentiles
pub const PERCENTILES: &[Percentile] = &[
    Percentile::p01,
    Percentile::p1,
    Percentile::p10,
    Percentile::p25,
    Percentile::p50,
    Percentile::p75,
    Percentile::p90,
    Percentile::p99,
    Percentile::Maximum,
];

pub struct Cpu {
    nanos_per_tick: u64,
    initialized: bool,
}

impl Cpu {
    /// creates a new `Cpu` `Sampler`
    pub fn new(_config: &Config) -> Self {
        Cpu {
            nanos_per_tick: crate::common::nanos_per_tick(),
            initialized: false,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
enum Category {
    User,
    Nice,
    System,
    Idle,
    Irq,
    Softirq,
    Steal,
    Guest,
    GuestNice,
}

impl Category {
    fn from_str(s: &str) -> Result<Category, ()> {
        match s {
            "user" => Ok(Category::User),
            "nice" => Ok(Category::Nice),
            "system" => Ok(Category::System),
            "idle" => Ok(Category::Idle),
            "irq" => Ok(Category::Irq),
            "softirq" => Ok(Category::Softirq),
            "steal" => Ok(Category::Steal),
            "guest" => Ok(Category::Guest),
            "guest_nice" => Ok(Category::GuestNice),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Category::User => write!(f, "user"),
            Category::Nice => write!(f, "nice"),
            Category::System => write!(f, "system"),
            Category::Idle => write!(f, "idle"),
            Category::Irq => write!(f, "irq"),
            Category::Softirq => write!(f, "softirq"),
            Category::Steal => write!(f, "steal"),
            Category::Guest => write!(f, "guest"),
            Category::GuestNice => write!(f, "guest_nice"),
        }
    }
}

struct ProcStat {
    cpu_total: HashMap<Category, u64>,
}

fn read_proc_stat() -> ProcStat {
    let file = File::open(PROC_STAT)
        .map_err(|e| debug!("could not read {}: {}", PROC_STAT, e))
        .unwrap();
    let mut file = BufReader::new(file);
    parse_proc_stat(&mut file)
}

fn parse_proc_stat<T: BufRead>(reader: &mut T) -> ProcStat {
    let mut ret = ProcStat {
        cpu_total: HashMap::new(),
    };
    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts[0] == "cpu" && parts.len() == 11 {
            ret.cpu_total
                .insert(Category::User, parts[1].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Category::Nice, parts[2].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Category::System, parts[3].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Category::Idle, parts[4].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Category::Irq, parts[6].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Category::Softirq, parts[7].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Category::Steal, parts[8].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Category::Guest, parts[9].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Category::GuestNice, parts[10].parse().unwrap_or(0));
        }
    }
    ret
}

impl Sampler for Cpu {
    fn name(&self) -> String {
        "cpu".to_string()
    }

    fn sample(&mut self, recorder: &Recorder<u32>, config: &Config) -> Result<(), SamplerError> {
        trace!("sample {}", self.name());
        let data = read_proc_stat();
        let time = time::precise_time_ns();
        if !self.initialized {
            self.register(recorder, config);
        }
        for statistic in config.cpu().statistics() {
            if let Ok(work) = Category::from_str(&statistic) {
                let raw = *data.cpu_total.get(&work).unwrap_or(&0);
                let value = raw * self.nanos_per_tick;
                record_counter(recorder, format!("cpu/{}", work), time, value);
            }
        }
        Ok(())
    }

    fn register(&mut self, recorder: &Recorder<u32>, config: &Config) {
        trace!("register {}", self.name());
        if !self.initialized {
            let cores = crate::common::hardware_threads().unwrap_or(1);

            for statistic in config.cpu().statistics() {
                if let Ok(work) = Category::from_str(&statistic) {
                    register_counter(
                        recorder,
                        format!("cpu/{}", work),
                        2 * cores * SECOND,
                        3,
                        config.general().interval(),
                        PERCENTILES,
                    );
                } else {
                    debug!("unknown statistic: {}", statistic);
                }
            }

            self.initialized = true;
        }
    }

    fn deregister(&mut self, recorder: &Recorder<u32>, config: &Config) {
        trace!("deregister {}", self.name());
        if self.initialized {
            for statistic in config.cpu().statistics() {
                if let Ok(work) = Category::from_str(&statistic) {
                    recorder.delete_channel(format!("cpu/{}", work));
                }
            }
            self.initialized = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proc_stat() {
        let file = File::open(&format!("tests/data{}", PROC_STAT))
            .map_err(|e| panic!("could not read file: {}", e))
            .unwrap();
        let mut file = BufReader::new(file);
        let data = parse_proc_stat(&mut file);
        assert_eq!(*data.cpu_total.get(&Category::User).unwrap_or(&0), 370627);
        assert_eq!(*data.cpu_total.get(&Category::Nice).unwrap_or(&0), 0);
        assert_eq!(*data.cpu_total.get(&Category::System).unwrap_or(&0), 64096);
        assert_eq!(*data.cpu_total.get(&Category::Idle).unwrap_or(&0), 8020800);
        assert_eq!(*data.cpu_total.get(&Category::Irq).unwrap_or(&0), 0);
        assert_eq!(*data.cpu_total.get(&Category::Softirq).unwrap_or(&0), 3053);
        assert_eq!(*data.cpu_total.get(&Category::Steal).unwrap_or(&0), 0);
        assert_eq!(*data.cpu_total.get(&Category::Guest).unwrap_or(&0), 0);
        assert_eq!(*data.cpu_total.get(&Category::GuestNice).unwrap_or(&0), 0);
    }
}
