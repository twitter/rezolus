// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::{Config, SamplerConfig};
use crate::samplers::Common;
use crate::Sampler;

use async_trait::async_trait;
use metrics::*;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::runtime::Handle;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Udp {
    common: Common,
}

#[async_trait]
impl Sampler for Udp {
    type Statistic = UdpStatistic;

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

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }

        debug!("sampling");
        self.register();

        // sample /proc/net/snmp
        if let Ok(snmp) = nested_map_from_file("/proc/net/snmp").await {
            let time = time::precise_time_ns();
            for statistic in self.sampler_config().statistics() {
                if let Some((pkey, lkey)) = statistic.keys() {
                    if let Some(inner) = snmp.get(pkey) {
                        if let Some(value) = inner.get(lkey) {
                            self.metrics().record_counter(statistic, time, *value);
                        }
                    }
                }
            }
        }

        // sample /proc/net/netstat
        if let Ok(snmp) = nested_map_from_file("/proc/net/netstat").await {
            let time = time::precise_time_ns();
            for statistic in self.sampler_config().statistics() {
                if let Some((pkey, lkey)) = statistic.keys() {
                    if let Some(inner) = snmp.get(pkey) {
                        if let Some(value) = inner.get(lkey) {
                            self.metrics().record_counter(statistic, time, *value);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().udp()
    }

    fn summary(&self, _statistic: &Self::Statistic) -> Option<Summary> {
        Some(Summary::histogram(
            1_000_000_000_000,
            3,
            Some(self.general_config().window()),
        ))
    }
}

async fn nested_map_from_file<T: AsRef<Path>>(
    path: T,
) -> Result<HashMap<String, HashMap<String, u64>>, std::io::Error> {
    let mut ret = HashMap::<String, HashMap<String, u64>>::new();
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    while let Some(keys) = lines.next_line().await? {
        while let Some(values) = lines.next_line().await? {
            let keys: Vec<&str> = keys.trim().split_whitespace().collect();
            let values: Vec<&str> = values.trim().split_whitespace().collect();
            if keys.len() > 2 {
                let pkey = keys[0];
                if !ret.contains_key(pkey) {
                    ret.insert(pkey.to_string(), Default::default());
                }
                let inner = ret.get_mut(&pkey.to_string()).unwrap();
                for (i, key) in keys.iter().enumerate().skip(1) {
                    let value: u64 = values.get(i).unwrap_or(&"0").parse().unwrap_or(0);
                    inner.insert((*key).to_string(), value);
                }
            }
        }
    }
    Ok(ret)
}
