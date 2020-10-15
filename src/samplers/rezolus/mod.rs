// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
use std::io::SeekFrom;

use async_trait::async_trait;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};

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
    proc_stat: Option<File>,
    proc_statm: Option<File>,
    statistics: Vec<RezolusStatistic>,
}

#[async_trait]
impl Sampler for Rezolus {
    type Statistic = RezolusStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let statistics = common.config().samplers().rezolus().statistics();
        let sampler = Self {
            common,
            nanos_per_tick: nanos_per_tick() as u64,
            proc_stat: None,
            proc_statm: None,
            statistics,
        };
        if sampler.sampler_config().enabled() {
            sampler.register();
        }
        Ok(sampler)
    }

    fn spawn(common: Common) {
        if common.config().samplers().rezolus().enabled() {
            if let Ok(mut sampler) = Self::new(common.clone()) {
                common.runtime().spawn(async move {
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
        let r = self.sample_memory().await;
        self.map_result(r)?;

        let r = self.sample_cpu().await;
        self.map_result(r)?;

        Ok(())
    }
}

impl Rezolus {
    async fn sample_cpu(&mut self) -> Result<(), std::io::Error> {
        if self.proc_stat.is_none() {
            let pid: u32 = std::process::id();
            let path = format!("/proc/{}/stat", pid);
            let file = File::open(path).await?;
            self.proc_stat = Some(file);
        }

        if let Some(file) = &mut self.proc_stat {
            file.seek(SeekFrom::Start(0)).await?;
            let mut reader = BufReader::new(file);
            let mut result = HashMap::new();
            let mut line = String::new();
            if reader.read_line(&mut line).await? > 0 {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let user = parts.get(13).map(|v| v.parse().unwrap_or(0)).unwrap_or(0)
                    + parts.get(15).map(|v| v.parse().unwrap_or(0)).unwrap_or(0);
                let system = parts.get(14).map(|v| v.parse().unwrap_or(0)).unwrap_or(0)
                    + parts.get(16).map(|v| v.parse().unwrap_or(0)).unwrap_or(0);
                result.insert(RezolusStatistic::CpuUser, user * self.nanos_per_tick);
                result.insert(RezolusStatistic::CpuSystem, system * self.nanos_per_tick);
                line.clear();
            }

            let time = Instant::now();
            for statistic in &self.statistics {
                if let Some(value) = result.get(statistic) {
                    let _ = self.metrics().record_counter(statistic, time, *value);
                }
            }
        }

        Ok(())
    }

    async fn sample_memory(&mut self) -> Result<(), std::io::Error> {
        if self.proc_statm.is_none() {
            let pid: u32 = std::process::id();
            let path = format!("/proc/{}/statm", pid);
            let file = File::open(path).await?;
            self.proc_statm = Some(file);
        }

        if let Some(file) = &mut self.proc_statm {
            file.seek(SeekFrom::Start(0)).await?;
            let mut result_memory = HashMap::new();
            let mut reader = BufReader::new(file);
            let mut line = String::new();
            if reader.read_line(&mut line).await? > 0 {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let vm = parts.get(0).map(|v| v.parse().unwrap_or(0)).unwrap_or(0);
                let rss = parts.get(1).map(|v| v.parse().unwrap_or(0)).unwrap_or(0);
                result_memory.insert(RezolusStatistic::MemoryVirtual, vm);
                result_memory.insert(RezolusStatistic::MemoryResident, rss);
                line.clear();
            }

            let time = Instant::now();
            for statistic in &self.statistics {
                if let Some(value) = result_memory.get(statistic) {
                    let _ = self.metrics().record_gauge(statistic, time, *value * 4096);
                }
            }
        }

        Ok(())
    }
}
