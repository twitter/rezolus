// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::{Config, SamplerConfig};
use crate::samplers::Common;
use crate::Sampler;
use async_trait::async_trait;
use metrics::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Handle;

use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

mod config;
mod stat;

pub use config::*;
pub use stat::*;

pub struct Softnet {
    common: Common,
}

#[async_trait]
impl Sampler for Softnet {
    type Statistic = SoftnetStatistic;
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
        self.common.config().softnet()
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

        let file = File::open("/proc/net/softnet_stat").await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut result = HashMap::new();

        while let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for statistic in self.sampler_config().statistics() {
                if !result.contains_key(statistic) {
                    result.insert(*statistic, 0);
                }
                let current = result.get_mut(statistic).unwrap();
                *current += u64::from_str_radix(parts[*statistic as usize], 16).unwrap_or(0);
            }
        }

        let time = time::precise_time_ns();
        for (stat, value) in result {
            self.metrics().record_counter(&stat, time, value);
        }

        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        Ok(())
    }

    fn summary(&self, _statistic: &Self::Statistic) -> Option<Summary> {
        Some(Summary::histogram(
            1_000_000_000_000,
            3,
            Some(self.general_config().window()),
        ))
    }
}
