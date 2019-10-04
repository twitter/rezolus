// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod statistics;

use self::statistics::Statistic;
use crate::common::*;
use crate::config::*;
use crate::samplers::{Common, Sampler};

use failure::Error;
use logger::*;
use metrics::{AtomicU32, Metrics, Percentile};
use time;

use std::collections::HashMap;
use std::path::Path;

pub struct Rezolus<'a> {
    common: Common<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// provides information about memory usage in pages
pub enum ProcessMemory {
    /// total program size
    Size,
    /// resident set size
    Resident,
    /// number of resident shared pages
    Shared,
    /// text (code)
    Text,
    /// data + stack
    Data,
}

fn parse_process_memory_stats<P: AsRef<Path>>(path: P) -> HashMap<ProcessMemory, u64> {
    let mut result = HashMap::new();
    let content = file::string_from_file(path).expect("failed to read statm");
    let tokens: Vec<&str> = content.split_whitespace().collect();
    if let Ok(size) = tokens[0].parse::<u64>() {
        result.insert(ProcessMemory::Size, size);
    }
    if let Ok(resident) = tokens[1].parse::<u64>() {
        result.insert(ProcessMemory::Resident, resident);
    }
    if let Ok(shared) = tokens[2].parse::<u64>() {
        result.insert(ProcessMemory::Shared, shared);
    }
    if let Ok(text) = tokens[3].parse::<u64>() {
        result.insert(ProcessMemory::Text, text);
    }
    if let Ok(data) = tokens[5].parse::<u64>() {
        result.insert(ProcessMemory::Data, data);
    }
    result
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// provides information about process
pub enum ProcessStat {
    /// Time scheduled in user mode in clock ticks
    UserTime,
    /// Time scheduled in kernel mode in clock ticks
    SystemTime,
    /// Time children have been scheduled in user mode in clock ticks
    ChildrenUserTime,
    /// Time children have been scheduled in kernel mode in clock ticks
    ChildrenSystemTime,
}

fn parse_process_stats<P: AsRef<Path>>(path: P) -> HashMap<ProcessStat, u64> {
    let mut result = HashMap::new();
    let content = file::string_from_file(path).expect("failed to read stat");
    let tokens: Vec<&str> = content.split_whitespace().collect();
    if let Ok(utime) = tokens[13].parse::<u64>() {
        result.insert(ProcessStat::UserTime, utime);
    }
    if let Ok(stime) = tokens[14].parse::<u64>() {
        result.insert(ProcessStat::SystemTime, stime);
    }
    if let Ok(cutime) = tokens[15].parse::<u64>() {
        result.insert(ProcessStat::ChildrenUserTime, cutime);
    }
    if let Ok(cstime) = tokens[16].parse::<u64>() {
        result.insert(ProcessStat::ChildrenSystemTime, cstime);
    }
    result
}

impl<'a> Rezolus<'a> {
    pub fn gauges(&self) -> Vec<String> {
        vec![
            Statistic::MemoryResident.to_string(),
            Statistic::MemoryVirtual.to_string(),
        ]
    }

    pub fn counters(&self) -> Vec<String> {
        vec![
            Statistic::CpuUser.to_string(),
            Statistic::CpuKernel.to_string(),
        ]
    }

    pub fn memory_usage(&self) {
        let time = time::precise_time_ns();
        let pid: u32 = std::process::id();
        let parsed = parse_process_memory_stats(format!("/proc/{}/statm", pid));
        self.common.record_gauge(
            &Statistic::MemoryVirtual,
            time,
            parsed.get(&ProcessMemory::Size).unwrap() * 4096,
        );
        self.common.record_gauge(
            &Statistic::MemoryResident,
            time,
            parsed.get(&ProcessMemory::Resident).unwrap() * 4096,
        );
    }

    pub fn cpu_usage(&self) {
        let time = time::precise_time_ns();
        let pid: u32 = std::process::id();
        let parsed = parse_process_stats(format!("/proc/{}/stat", pid));

        let user_seconds = (*parsed.get(&ProcessStat::UserTime).unwrap_or(&0)
            + *parsed.get(&ProcessStat::ChildrenUserTime).unwrap_or(&0))
            * nanos_per_tick();
        self.common
            .record_counter(&Statistic::CpuUser, time, user_seconds);

        let kernel_seconds = (*parsed.get(&ProcessStat::SystemTime).unwrap_or(&0)
            + *parsed.get(&ProcessStat::ChildrenSystemTime).unwrap_or(&0))
            * nanos_per_tick();
        self.common
            .record_counter(&Statistic::CpuKernel, time, kernel_seconds);
    }
}

impl<'a> Sampler<'a> for Rezolus<'a> {
    fn new(
        config: &'a Config,
        metrics: &'a Metrics<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
        Ok(Some(Box::new(Rezolus {
            common: Common::new(config, metrics),
        })))
    }

    fn common(&self) -> &Common<'a> {
        &self.common
    }

    fn name(&self) -> String {
        "rezolus".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        trace!("sample {}", self.name());
        self.register();
        self.memory_usage();
        self.cpu_usage();
        Ok(())
    }

    fn interval(&self) -> usize {
        self.common().config().interval()
    }

    fn register(&mut self) {
        if !self.common.initialized() {
            trace!("register {}", self.name());
            for label in self.gauges() {
                self.common
                    .register_gauge(&label, 32 * TERABYTE, 3, &[Percentile::Maximum]);
            }
            for label in self.counters() {
                self.common
                    .register_counter(&label, BILLION, 3, &[Percentile::Maximum]);
            }
            self.common.set_initialized(true)
        }
    }

    fn deregister(&mut self) {
        if self.common.initialized() {
            trace!("deregister {}", self.name());
            for label in self.gauges() {
                self.common.delete_channel(&label);
            }
            for label in self.counters() {
                self.common.delete_channel(&label);
            }
            self.common.set_initialized(false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_memory_stats() {
        let parsed = parse_process_memory_stats("tests/data/proc/1000/statm");
        assert_eq!(parsed.get(&ProcessMemory::Size), Some(&149100));
        assert_eq!(parsed.get(&ProcessMemory::Resident), Some(&34107));
        assert_eq!(parsed.get(&ProcessMemory::Shared), Some(&19859));
        assert_eq!(parsed.get(&ProcessMemory::Text), Some(&29790));
        assert_eq!(parsed.get(&ProcessMemory::Data), Some(&16385));
    }

    #[test]
    fn process_stats() {
        let parsed = parse_process_stats("tests/data/proc/1000/stat");
        assert_eq!(parsed.get(&ProcessStat::UserTime), Some(&1104933));
        assert_eq!(parsed.get(&ProcessStat::SystemTime), Some(&2789797));
        assert_eq!(parsed.get(&ProcessStat::ChildrenUserTime), Some(&0));
        assert_eq!(parsed.get(&ProcessStat::ChildrenSystemTime), Some(&0));
    }
}
