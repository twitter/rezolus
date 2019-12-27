// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::{Config, SamplerConfig};
use crate::samplers::Common;
use crate::Sampler;
use async_trait::async_trait;
use metrics::*;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs::File;

use tokio::prelude::*;
use tokio::runtime::Handle;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

pub struct Cpuidle {
    common: Common,
}

#[async_trait]
impl Sampler for Cpuidle {
    type Statistic = CpuidleStatistic;

    fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Result<Self, failure::Error> {
        Ok(Self {
            common: Common::new(config, metrics),
        })
    }

    fn spawn(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>, handle: &Handle) {
        if let Ok(mut sampler) = Self::new(config.clone(), metrics) {
            handle.spawn(async move {
                loop {
                    let _ = sampler.sample().await;
                }
            });
        } else if !config.fault_tolerant() {
            fatal!("failed to initialize sampler");
        } else {
            error!("failed to initialize sampler");
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().cpuidle()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if !self.sampler_config().enabled() {
            if let Some(ref mut delay) = self.delay() {
                delay.tick().await;
            }

            return Ok(());
        }

        debug!("sampling");
        self.register();

        self.sample_cpuidle().await?;

        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        Ok(())
    }

    fn summary(&self, _statistic: &Self::Statistic) -> Option<Summary> {
        let max = crate::common::hardware_threads().unwrap_or(1024) * 2_000_000_000;
        Some(Summary::histogram(
            max,
            3,
            Some(self.general_config().window()),
        ))
    }
}

impl Cpuidle {
    async fn sample_cpuidle(&self) -> Result<(), std::io::Error> {
        let mut result = HashMap::<CpuidleStatistic, u64>::new();

        // iterate through all cpus
        let cpu_regex = Regex::new(r"^cpu\d+$").unwrap();
        let state_regex = Regex::new(r"^state\d+$").unwrap();
        let mut cpu_dir = tokio::fs::read_dir("/sys/devices/system/cpu").await?;
        while let Some(cpu_entry) = cpu_dir.next_entry().await? {
            if let Ok(cpu_name) = cpu_entry.file_name().into_string() {
                if cpu_regex.is_match(&cpu_name) {
                    // iterate through all cpuidle states
                    let cpuidle_dir = format!("/sys/devices/system/cpu/{}/cpuidle", cpu_name);
                    let mut cpuidle_dir = tokio::fs::read_dir(cpuidle_dir).await?;
                    while let Some(cpuidle_entry) = cpuidle_dir.next_entry().await? {
                        if let Ok(cpuidle_name) = cpuidle_entry.file_name().into_string() {
                            if state_regex.is_match(&cpuidle_name) {
                                // have an actual state here

                                // get the name of the state
                                let name_file = format!(
                                    "/sys/devices/system/cpu/{}/cpuidle/{}/name",
                                    cpu_name, cpuidle_name
                                );
                                let mut name_file = File::open(name_file).await?;
                                let mut name_content = Vec::new();
                                name_file.read_to_end(&mut name_content).await?;
                                if let Ok(name_string) = std::str::from_utf8(&name_content) {
                                    if let Ok(state) = name_string.parse() {
                                        // get the time spent in the state
                                        let time_file = format!(
                                            "/sys/devices/system/cpu/{}/cpuidle/{}/time",
                                            cpu_name, cpuidle_name
                                        );
                                        let mut time_file = File::open(time_file).await?;
                                        let mut time_content = Vec::new();
                                        time_file.read_to_end(&mut time_content).await?;
                                        if let Ok(time_string) = std::str::from_utf8(&time_content)
                                        {
                                            if let Ok(time) = time_string.parse::<u64>() {
                                                let counter = result
                                                    .entry(CpuidleStatistic::Time(state))
                                                    .or_insert(0);
                                                *counter += time;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let time = time::precise_time_ns();
        for stat in self.sampler_config().statistics() {
            if let Some(value) = result.get(stat) {
                self.metrics().record_counter(stat, time, *value);
            }
        }

        Ok(())
    }
}
