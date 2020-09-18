// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
use std::io::SeekFrom;
use std::time::*;

use async_trait::async_trait;

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
    softnet_stat: Option<File>,
    statistics: Vec<SoftnetStatistic>,
}

#[async_trait]
impl Sampler for Softnet {
    type Statistic = SoftnetStatistic;
    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let statistics = common.config().samplers().softnet().statistics();
        let sampler = Self {
            common,
            softnet_stat: None,
            statistics,
        };
        if sampler.sampler_config().enabled() {
            sampler.register();
        }
        Ok(sampler)
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

        let r = self.sample_softnet_stats().await;
        self.map_result(r)?;

        Ok(())
    }
}

impl Softnet {
    async fn sample_softnet_stats(&mut self) -> Result<(), std::io::Error> {
        if self.softnet_stat.is_none() {
            let file = File::open("/proc/net/softnet_stat").await?;
            self.softnet_stat = Some(file);
        }

        if let Some(file) = &mut self.softnet_stat {
            file.seek(SeekFrom::Start(0)).await?;
            let mut reader = BufReader::new(file);
            let mut line = String::new();
            let mut result = HashMap::<SoftnetStatistic, u64>::new();

            while reader.read_line(&mut line).await? > 0 {
                for (id, part) in line.split_whitespace().enumerate() {
                    if let Some(statistic) = num::FromPrimitive::from_usize(id) {
                        if !result.contains_key(&statistic) {
                            result.insert(statistic, 0);
                        }
                        let current = result.get_mut(&statistic).unwrap();
                        *current += u64::from_str_radix(part, 16).unwrap_or(0);
                    }
                }
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
}
