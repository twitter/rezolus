// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod statistics;

use self::statistics::Statistic;
use crate::common::*;
use crate::config::*;
use crate::samplers::{Common, Sampler};

use failure::*;
use logger::*;
use metrics::*;
use time;

use std::io::{BufRead, BufReader};
use std::sync::Arc;

pub struct Container {
    common: Common,
    cgroup: Option<String>,
    nanos_per_tick: u64,
}

impl Sampler for Container {
    fn new(
        config: Arc<Config>,
        metrics: Metrics<AtomicU32>,
    ) -> Result<Option<Box<dyn Sampler>>, Error> {
        if config.container().enabled() {
            let mut cgroup = None;
            let path = format!("/proc/{}/cgroup", std::process::id());
            let file = std::fs::File::open(&path)
                .map_err(|e| format_err!("failed to open file ({:?}): {}", path, e))?;
            let f = BufReader::new(file);
            for line in f.lines() {
                let line = line.unwrap();
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 3 && parts[1] == "cpu,cpuacct" {
                    cgroup = Some(parts[2].to_string());
                }
            }
            if cgroup.is_some() {
                Ok(Some(Box::new(Container {
                    common: Common::new(config, metrics),
                    cgroup,
                    nanos_per_tick: crate::common::nanos_per_tick(),
                }) as Box<dyn Sampler>))
            } else {
                Err(format_err!("failed to find cgroup"))
            }
        } else {
            Ok(None)
        }
    }

    fn interval(&self) -> usize {
        self.common()
            .config()
            .container()
            .interval()
            .unwrap_or_else(|| self.common().config().interval())
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn name(&self) -> String {
        "container".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        // gather current state
        trace!("sampling {}", self.name());
        self.register();
        let time = time::precise_time_ns();

        let mut total = 0;

        let path = format!(
            "/sys/fs/cgroup/cpu,cpuacct{}/cpuacct.stat",
            self.cgroup.as_ref().unwrap()
        );
        if let Ok(file) = std::fs::File::open(&path) {
            let file = BufReader::new(file);
            for line in file.lines() {
                if let Ok(line) = line {
                    let parts: Vec<&str> = line.split(' ').collect();
                    if parts.len() == 2 {
                        match parts[0] {
                            "system" => {
                                if let Ok(ticks) = parts[1].parse::<u64>() {
                                    let ns = ticks * self.nanos_per_tick;
                                    self.common.record_counter(&Statistic::CpuSystem, time, ns);
                                    total += ns;
                                }
                            }
                            "user" => {
                                if let Ok(ticks) = parts[1].parse::<u64>() {
                                    let ns = ticks * self.nanos_per_tick;
                                    self.common.record_counter(&Statistic::CpuUser, time, ns);
                                    total += ns;
                                }
                            }
                            &_ => {}
                        }
                    }
                }
            }
        }

        if total != 0 {
            self.common
                .record_counter(&Statistic::CpuTotal, time, total);
        }

        Ok(())
    }

    fn register(&mut self) {
        if !self.common.initialized() {
            let cores = crate::common::hardware_threads().unwrap_or(1);
            for statistic in &[
                Statistic::CpuSystem,
                Statistic::CpuTotal,
                Statistic::CpuUser,
            ] {
                self.common
                    .register_counter(statistic, 2 * cores * SECOND, 3, PERCENTILES);
            }
            self.common.set_initialized(true);
        }
    }

    fn deregister(&mut self) {
        for statistic in &[
            Statistic::CpuSystem,
            Statistic::CpuTotal,
            Statistic::CpuUser,
        ] {
            self.common.delete_channel(statistic);
        }
        self.common.set_initialized(false);
    }
}
