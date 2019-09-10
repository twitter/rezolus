// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::*;
use crate::config::Config;
use crate::samplers::{Common, Sampler};

use failure::*;
use logger::*;
use metrics::*;
use time;

use std::io::{BufRead, BufReader};

pub struct Container<'a> {
    common: Common<'a>,
    cgroup: Option<String>,
    initialized: bool,
    nanos_per_tick: u64,
}

impl<'a> Sampler<'a> for Container<'a> {
    fn new(
        config: &'a Config,
        recorder: &'a Recorder<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
        if config.container().enabled() {
            let mut cgroup = None;
            let path = format!("/proc/{}/cgroup", std::process::id());
            let file = std::fs::File::open(&path)
                .map_err(|e| format_err!("failed to open file ({:?}): {}", path, e))?;
            let f = BufReader::new(file);
            for line in f.lines() {
                let line = line.unwrap();
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 3 {
                    if parts[1] == "cpu,cpuacct" {
                        cgroup = Some(parts[2].to_string());
                    }
                }
            }
            if cgroup.is_some() {
                Ok(Some(Box::new(Container {
                    common: Common::new(config, recorder),
                    cgroup,
                    initialized: false,
                    nanos_per_tick: crate::common::nanos_per_tick(),
                })))
            } else {
                Err(format_err!("failed to find cgroup"))
            }
        } else {
            Ok(None)
        }
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
                                    self.common
                                        .record_counter(&"container/cpu/system", time, ns);
                                    total += ns;
                                }
                            }
                            "user" => {
                                if let Ok(ticks) = parts[1].parse::<u64>() {
                                    let ns = ticks * self.nanos_per_tick;
                                    self.common.record_counter(&"container/cpu/user", time, ns);
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
                .record_counter(&"container/cpu/total", time, total);
        }

        Ok(())
    }

    fn register(&mut self) {
        if !self.initialized {
            let cores = crate::common::hardware_threads().unwrap_or(1);
            self.common.register_counter(
                &"container/cpu/total",
                2 * cores * SECOND,
                3,
                PERCENTILES,
            );
            self.common.register_counter(
                &"container/cpu/system",
                2 * cores * SECOND,
                3,
                PERCENTILES,
            );
            self.common
                .register_counter(&"container/cpu/user", 2 * cores * SECOND, 3, PERCENTILES);
            self.initialized = true;
        }
    }

    fn deregister(&mut self) {
        if self.initialized {
            self.common.delete_channel(&"container/cpu/total");
            self.common.delete_channel(&"container/cpu/system");
            self.common.delete_channel(&"container/cpu/user");
            self.initialized = false;
        }
    }
}
