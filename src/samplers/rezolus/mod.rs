// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::Config;
use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::Sampler;
use async_trait::async_trait;
use atomics::AtomicU32;
use metrics::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::runtime::Handle;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

pub fn nanos_per_tick() -> u64 {
    let ticks_per_second = sysconf::raw::sysconf(sysconf::raw::SysconfVariable::ScClkTck)
        .expect("Failed to get Clock Ticks per Second") as u64;
    1_000_000_000 / ticks_per_second
}

pub struct Rezolus {
    common: Common,
    nanos_per_tick: u64,
}

#[async_trait]
impl Sampler for Rezolus {
    type Statistic = RezolusStatistic;

    fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Result<Self, failure::Error> {
        Ok(Self {
            common: Common::new(config, metrics),
            nanos_per_tick: nanos_per_tick() as u64,
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
        self.common.config().samplers().rezolus()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if !self.sampler_config().enabled() {
            if let Some(ref mut delay) = self.delay() {
                delay.tick().await;
            }

            return Ok(());
        }

        debug!("sampling");
        self.sample_memory().await?;
        self.sample_cpu().await?;

        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        Ok(())
    }
}

impl Rezolus {
    async fn sample_cpu(&self) -> Result<(), std::io::Error> {
        self.register();
        let pid: u32 = std::process::id();
        let mut result = HashMap::new();
        let path = format!("/proc/{}/stat", pid);
        let file = File::open(path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let user = parts[13].parse().unwrap_or(0) + parts[15].parse().unwrap_or(0);
            let system = parts[14].parse().unwrap_or(0) + parts[16].parse().unwrap_or(0);
            result.insert(RezolusStatistic::CpuUser, user * self.nanos_per_tick);
            result.insert(RezolusStatistic::CpuSystem, system * self.nanos_per_tick);
        }

        let time = time::precise_time_ns();
        for (stat, value) in result {
            debug!("record: {} value: {}", stat.name(), value);
            self.metrics().record_counter(&stat, time, value);
        }
        Ok(())
    }

    async fn sample_memory(&self) -> Result<(), std::io::Error> {
        let pid: u32 = std::process::id();
        let mut result_memory = HashMap::new();
        let path = format!("/proc/{}/statm", pid);
        let file = File::open(path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let vm = parts[0].parse().unwrap_or(0);
            let rss = parts[1].parse().unwrap_or(0);
            result_memory.insert(RezolusStatistic::MemoryVirtual, vm);
            result_memory.insert(RezolusStatistic::MemoryResident, rss);
        }

        let time = time::precise_time_ns();
        for (stat, value) in result_memory {
            self.metrics().record_gauge(&stat, time, value * 4096);
        }
        Ok(())
    }
}
