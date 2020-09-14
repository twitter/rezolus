// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;

use async_trait::async_trait;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::common::*;
use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::Sampler;
use std::time::*;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

pub fn nanos_per_tick() -> u64 {
    let ticks_per_second = sysconf::raw::sysconf(sysconf::raw::SysconfVariable::ScClkTck)
        .expect("Failed to get Clock Ticks per Second") as u64;
    SECOND / ticks_per_second
}

pub struct Rezolus {
    common: Common,
    nanos_per_tick: u64,
}

#[async_trait]
impl Sampler for Rezolus {
    type Statistic = RezolusStatistic;

    fn new(common: Common) -> Result<Self, failure::Error> {
        Ok(Self {
            common,
            nanos_per_tick: nanos_per_tick() as u64,
        })
    }

    fn spawn(common: Common) {
        if let Ok(mut sampler) = Self::new(common.clone()) {
            common.handle.spawn(async move {
                loop {
                    let _ = sampler.sample().await;
                }
            });
        } else if !common.config.fault_tolerant() {
            fatal!("failed to initialize rezolus sampler");
        } else {
            error!("failed to initialize rezolus sampler");
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
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }

        debug!("sampling");
        self.map_result(self.sample_memory().await)?;
        self.map_result(self.sample_cpu().await)?;

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
            let user = parts.get(13).map(|v| v.parse().unwrap_or(0)).unwrap_or(0)
                + parts.get(15).map(|v| v.parse().unwrap_or(0)).unwrap_or(0);
            let system = parts.get(14).map(|v| v.parse().unwrap_or(0)).unwrap_or(0)
                + parts.get(16).map(|v| v.parse().unwrap_or(0)).unwrap_or(0);
            result.insert(RezolusStatistic::CpuUser, user * self.nanos_per_tick);
            result.insert(RezolusStatistic::CpuSystem, system * self.nanos_per_tick);
        }

        let time = Instant::now();
        for (stat, value) in result {
            let _ = self.metrics().record_counter(&stat, time, value);
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
            let vm = parts.get(0).map(|v| v.parse().unwrap_or(0)).unwrap_or(0);
            let rss = parts.get(1).map(|v| v.parse().unwrap_or(0)).unwrap_or(0);
            result_memory.insert(RezolusStatistic::MemoryVirtual, vm);
            result_memory.insert(RezolusStatistic::MemoryResident, rss);
        }

        let time = Instant::now();
        for (stat, value) in result_memory {
            let _ = self.metrics().record_gauge(&stat, time, value * 4096);
        }
        Ok(())
    }
}
