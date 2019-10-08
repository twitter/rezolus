// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub(crate) mod statistics;

use self::statistics::*;
use crate::common::*;
use crate::config::*;
use crate::samplers::{Common, Sampler};

use failure::Error;
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

pub struct Cpu<'a> {
    common: Common<'a>,
    nanos_per_tick: u64,
}

struct ProcStat {
    cpu_total: HashMap<Statistic, u64>,
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
                .insert(Statistic::User, parts[1].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Statistic::Nice, parts[2].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Statistic::System, parts[3].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Statistic::Idle, parts[4].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Statistic::Irq, parts[6].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Statistic::Softirq, parts[7].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Statistic::Steal, parts[8].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Statistic::Guest, parts[9].parse().unwrap_or(0));
            ret.cpu_total
                .insert(Statistic::GuestNice, parts[10].parse().unwrap_or(0));
        }
    }
    ret
}

impl<'a> Sampler<'a> for Cpu<'a> {
    fn new(
        config: &'a Config,
        metrics: &'a Metrics<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
        if config.cpu().enabled() {
            Ok(Some(Box::new(Cpu {
                common: Common::new(config, metrics),
                nanos_per_tick: crate::common::nanos_per_tick(),
            })))
        } else {
            Ok(None)
        }
    }

    fn common(&self) -> &Common<'a> {
        &self.common
    }

    fn name(&self) -> String {
        "cpu".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        trace!("sample {}", self.name());
        let data = read_proc_stat();
        let time = time::precise_time_ns();
        self.register();
        for statistic in self.common.config().cpu().statistics() {
            let raw = *data.cpu_total.get(&statistic).unwrap_or(&0);
            let value = raw * self.nanos_per_tick;
            self.common.record_counter(&statistic, time, value);
        }
        Ok(())
    }

    fn interval(&self) -> usize {
        self.common()
            .config()
            .cpu()
            .interval()
            .unwrap_or_else(|| self.common().config().interval())
    }

    fn register(&mut self) {
        trace!("register {}", self.name());
        if !self.common.initialized() {
            let cores = crate::common::hardware_threads().unwrap_or(1);

            for statistic in self.common.config().cpu().statistics() {
                self.common
                    .register_counter(&statistic, 2 * cores * SECOND, 3, PERCENTILES);
            }

            self.common.set_initialized(true);
        }
    }

    fn deregister(&mut self) {
        trace!("deregister {}", self.name());
        if self.common.initialized() {
            for statistic in self.common.config().cpu().statistics() {
                self.common.delete_channel(&statistic);
            }
            self.common.set_initialized(false);
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
        assert_eq!(*data.cpu_total.get(&Statistic::User).unwrap_or(&0), 370627);
        assert_eq!(*data.cpu_total.get(&Statistic::Nice).unwrap_or(&0), 0);
        assert_eq!(*data.cpu_total.get(&Statistic::System).unwrap_or(&0), 64096);
        assert_eq!(*data.cpu_total.get(&Statistic::Idle).unwrap_or(&0), 8020800);
        assert_eq!(*data.cpu_total.get(&Statistic::Irq).unwrap_or(&0), 0);
        assert_eq!(*data.cpu_total.get(&Statistic::Softirq).unwrap_or(&0), 3053);
        assert_eq!(*data.cpu_total.get(&Statistic::Steal).unwrap_or(&0), 0);
        assert_eq!(*data.cpu_total.get(&Statistic::Guest).unwrap_or(&0), 0);
        assert_eq!(*data.cpu_total.get(&Statistic::GuestNice).unwrap_or(&0), 0);
    }
}
