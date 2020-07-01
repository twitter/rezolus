// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_trait::async_trait;
use rustcommon_metrics::*;

use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::Sampler;

mod config;
mod stat;

pub use config::UdpConfig;
pub use stat::UdpStatistic;

#[allow(dead_code)]
pub struct Udp {
    common: Common,
}

#[async_trait]
impl Sampler for Udp {
    type Statistic = UdpStatistic;

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
            fatal!("failed to initialize udp sampler");
        } else {
            error!("failed to initialize udp sampler");
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

        self.map_result(self.sample_snmp().await)?;
        self.map_result(self.sample_netstat().await)?;

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

impl Udp {
    async fn sample_snmp(&self) -> Result<(), std::io::Error> {
        let snmp = crate::common::nested_map_from_file("/proc/net/snmp").await?;
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
        Ok(())
    }

    async fn sample_netstat(&self) -> Result<(), std::io::Error> {
        let netstat = crate::common::nested_map_from_file("/proc/net/netstat").await?;
        let time = time::precise_time_ns();
        for statistic in self.sampler_config().statistics() {
            if let Some((pkey, lkey)) = statistic.keys() {
                if let Some(inner) = netstat.get(pkey) {
                    if let Some(value) = inner.get(lkey) {
                        self.metrics().record_counter(statistic, time, *value);
                    }
                }
            }
        }
        Ok(())
    }
}
