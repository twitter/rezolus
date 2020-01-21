// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::*;
use crate::config::{Config, SamplerConfig};
use crate::samplers::Common;
use crate::Sampler;
use async_trait::async_trait;
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

use stat::MemoryStatistic as Stat;

#[allow(dead_code)]
pub struct Memory {
    common: Common,
}

#[async_trait]
impl Sampler for Memory {
    type Statistic = MemoryStatistic;

    fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Result<Self, failure::Error> {
        debug!("initializing");

        debug!("initialization complete");
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
        self.common.config().samplers().memory()
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

        self.sample_meminfo().await?;

        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        Ok(())
    }

    fn summary(&self, _statistic: &Self::Statistic) -> Option<Summary> {
        Some(Summary::histogram(
            TEBIBYTE,
            3,
            Some(self.general_config().window()),
        ))
    }
}

impl Memory {
    async fn sample_meminfo(&self) -> Result<(), std::io::Error> {
        let file = File::open("/proc/meminfo").await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut result = HashMap::new();

        while let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let label: String = parts[0].to_owned();
            let value: u64 = parts[1].parse().unwrap_or(0);
            let parts: Vec<&str> = label.split(':').collect();
            if let Some(stat) = match parts[0] {
                "MemTotal" => Some(Stat::Total),
                "MemFree" => Some(Stat::Free),
                "MemAvailable" => Some(Stat::Available),
                "Buffers" => Some(Stat::Buffers),
                "Cached" => Some(Stat::Cached),
                "SwapCached" => Some(Stat::SwapCached),
                "Active" => Some(Stat::Active),
                "Inactive" => Some(Stat::Inactive),
                "Active(anon)" => Some(Stat::ActiveAnon),
                "Inactive(anon)" => Some(Stat::InactiveAnon),
                "Unevictable" => Some(Stat::Unevictable),
                "Mlocked" => Some(Stat::Mlocked),
                "SwapTotal" => Some(Stat::SwapTotal),
                "SwapFree" => Some(Stat::SwapFree),
                "Dirty" => Some(Stat::Dirty),
                "Writeback" => Some(Stat::Writeback),
                "AnonPages" => Some(Stat::AnonPages),
                "Mapped" => Some(Stat::Mapped),
                "Shmem" => Some(Stat::Shmem),
                "Slab" => Some(Stat::SlabTotal),
                "SReclaimable" => Some(Stat::SlabReclaimable),
                "SUnreclaim" => Some(Stat::SlabUnreclaimable),
                "KernelStack" => Some(Stat::KernelStack),
                "PageTables" => Some(Stat::PageTables),
                "NFS_Unstable" => Some(Stat::NFSUnstable),
                "Bounce" => Some(Stat::Bounce),
                "WritebackTmp" => Some(Stat::WritebackTmp),
                "CommitLimit" => Some(Stat::CommitLimit),
                "Committed_AS" => Some(Stat::CommittedAS),
                "VmallocTotal" => Some(Stat::VmallocTotal),
                "VmallocUsed" => Some(Stat::VmallocUsed),
                "VmallocChunk" => Some(Stat::VmallocChunk),
                "Percpu" => Some(Stat::Percpu),
                "HardwareCorrupted" => Some(Stat::HardwareCorrupted),
                "AnonHugePages" => Some(Stat::AnonHugePages),
                "ShmemHugePages" => Some(Stat::ShmemHugePages),
                "ShmemPmdMapped" => Some(Stat::ShmemPmdMapped),
                "HugePages_Total" => Some(Stat::HugePagesTotal),
                "HugePages_Free" => Some(Stat::HugePagesFree),
                "HugePages_Rsvd" => Some(Stat::HugePagesRsvd),
                "HugePages_Surp" => Some(Stat::HugePagesSurp),
                "Hugepagesize" => Some(Stat::Hugepagesize),
                "Hugetlb" => Some(Stat::Hugetlb),
                "DirectMap4k" => Some(Stat::DirectMap4k),
                "DirectMap2M" => Some(Stat::DirectMap2M),
                "DirectMap1G" => Some(Stat::DirectMap1G),
                _ => None,
            } {
                result.insert(stat, value);
            }
        }

        let time = time::precise_time_ns();
        for stat in self.sampler_config().statistics() {
            if let Some(value) = result.get(stat) {
                self.metrics().record_gauge(stat, time, *value);
            }
        }
        Ok(())
    }
}
