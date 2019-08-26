// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::*;
use crate::config::Config;
use crate::samplers::{Sampler, Statistic};
use crate::stats::{record_counter, register_counter};

use failure::Error;
use logger::*;
use metrics::*;
use serde_derive::*;
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

pub struct Cpu<'a> {
    config: &'a Config,
    nanos_per_tick: u64,
    initialized: bool,
    recorder: &'a Recorder<AtomicU32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum CpuStatistic {
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

impl std::fmt::Display for CpuStatistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CpuStatistic::User => write!(f, "cpu/user"),
            CpuStatistic::Nice => write!(f, "cpu/nice"),
            CpuStatistic::System => write!(f, "cpu/system"),
            CpuStatistic::Idle => write!(f, "cpu/idle"),
            CpuStatistic::Irq => write!(f, "cpu/irq"),
            CpuStatistic::Softirq => write!(f, "cpu/softirq"),
            CpuStatistic::Steal => write!(f, "cpu/steal"),
            CpuStatistic::Guest => write!(f, "cpu/guest"),
            CpuStatistic::GuestNice => write!(f, "cpu/guest_nice"),
        }
    }
}

impl Statistic for CpuStatistic {}

struct ProcStat {
    cpu_total: HashMap<CpuStatistic, u64>,
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
                .insert(CpuStatistic::User, parts[1].parse().unwrap_or(0));
            ret.cpu_total
                .insert(CpuStatistic::Nice, parts[2].parse().unwrap_or(0));
            ret.cpu_total
                .insert(CpuStatistic::System, parts[3].parse().unwrap_or(0));
            ret.cpu_total
                .insert(CpuStatistic::Idle, parts[4].parse().unwrap_or(0));
            ret.cpu_total
                .insert(CpuStatistic::Irq, parts[6].parse().unwrap_or(0));
            ret.cpu_total
                .insert(CpuStatistic::Softirq, parts[7].parse().unwrap_or(0));
            ret.cpu_total
                .insert(CpuStatistic::Steal, parts[8].parse().unwrap_or(0));
            ret.cpu_total
                .insert(CpuStatistic::Guest, parts[9].parse().unwrap_or(0));
            ret.cpu_total
                .insert(CpuStatistic::GuestNice, parts[10].parse().unwrap_or(0));
        }
    }
    ret
}

impl<'a> Sampler<'a> for Cpu<'a> {
    fn new(
        config: &'a Config,
        recorder: &'a Recorder<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
        if config.cpu().enabled() {
            Ok(Some(Box::new(Cpu {
                config,
                nanos_per_tick: crate::common::nanos_per_tick(),
                initialized: false,
                recorder,
            })))
        } else {
            Ok(None)
        }
    }
    fn name(&self) -> String {
        "cpu".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        trace!("sample {}", self.name());
        let data = read_proc_stat();
        let time = time::precise_time_ns();
        if !self.initialized {
            self.register();
        }
        for statistic in self.config.cpu().statistics() {
            let raw = *data.cpu_total.get(&statistic).unwrap_or(&0);
            let value = raw * self.nanos_per_tick;
            record_counter(self.recorder, statistic, time, value);
        }
        Ok(())
    }

    fn register(&mut self) {
        trace!("register {}", self.name());
        if !self.initialized {
            let cores = crate::common::hardware_threads().unwrap_or(1);

            for statistic in self.config.cpu().statistics() {
                register_counter(
                    self.recorder,
                    statistic,
                    2 * cores * SECOND,
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
            for statistic in self.config.cpu().statistics() {
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
    fn test_parse_proc_stat() {
        let file = File::open(&format!("tests/data{}", PROC_STAT))
            .map_err(|e| panic!("could not read file: {}", e))
            .unwrap();
        let mut file = BufReader::new(file);
        let data = parse_proc_stat(&mut file);
        assert_eq!(
            *data.cpu_total.get(&CpuStatistic::User).unwrap_or(&0),
            370627
        );
        assert_eq!(*data.cpu_total.get(&CpuStatistic::Nice).unwrap_or(&0), 0);
        assert_eq!(
            *data.cpu_total.get(&CpuStatistic::System).unwrap_or(&0),
            64096
        );
        assert_eq!(
            *data.cpu_total.get(&CpuStatistic::Idle).unwrap_or(&0),
            8020800
        );
        assert_eq!(*data.cpu_total.get(&CpuStatistic::Irq).unwrap_or(&0), 0);
        assert_eq!(
            *data.cpu_total.get(&CpuStatistic::Softirq).unwrap_or(&0),
            3053
        );
        assert_eq!(*data.cpu_total.get(&CpuStatistic::Steal).unwrap_or(&0), 0);
        assert_eq!(*data.cpu_total.get(&CpuStatistic::Guest).unwrap_or(&0), 0);
        assert_eq!(
            *data.cpu_total.get(&CpuStatistic::GuestNice).unwrap_or(&0),
            0
        );
    }
}
