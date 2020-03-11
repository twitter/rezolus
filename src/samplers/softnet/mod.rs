// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;

use async_trait::async_trait;
use metrics::*;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::Sampler;

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
    fn new(common: Common) -> Result<Self, failure::Error> {
        Ok(Self { common })
    }

    fn spawn(common: Common) {
        if let Ok(mut sampler) = Self::new(common.clone()) {
            common.handle.spawn(async move {
                loop {
                    let _ = sampler.sample().await;
                }
            });
        } else if !common.config.fault_tolerant() {
            fatal!("failed to initialize softnet sampler");
        } else {
            error!("failed to initialize softnet sampler");
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().softnet()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
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
                *current += parts
                    .get(*statistic as usize)
                    .map(|v| u64::from_str_radix(v, 16).unwrap_or(0))
                    .unwrap_or(0);
            }
        }

        let time = time::precise_time_ns();
        for (stat, value) in result {
            self.metrics().record_counter(&stat, time, value);
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
