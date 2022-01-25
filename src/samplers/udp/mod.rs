// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_trait::async_trait;
use tokio::fs::File;

use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::*;

mod config;
mod stat;

pub use config::UdpConfig;
pub use stat::UdpStatistic;

#[allow(dead_code)]
pub struct Udp {
    common: Common,
    proc_net_snmp: Option<File>,
    proc_net_netstat: Option<File>,
    statistics: Vec<UdpStatistic>,
}

#[async_trait]
impl Sampler for Udp {
    type Statistic = UdpStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let statistics = common.config().samplers().udp().statistics();

        let sampler = Self {
            common,
            proc_net_snmp: None,
            proc_net_netstat: None,
            statistics,
        };
        if sampler.sampler_config().enabled() {
            sampler.register();
        }
        Ok(sampler)
    }

    fn spawn(common: Common) {
        if common.config().samplers().udp().enabled() {
            if let Ok(mut sampler) = Self::new(common.clone()) {
                common.runtime().spawn(async move {
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

        let r = self.sample_snmp().await;
        self.map_result(r)?;

        let r = self.sample_netstat().await;
        self.map_result(r)?;

        Ok(())
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().udp()
    }
}

impl Udp {
    async fn sample_snmp(&mut self) -> Result<(), std::io::Error> {
        if self.proc_net_snmp.is_none() {
            let file = File::open("/proc/net/snmp").await?;
            self.proc_net_snmp = Some(file);
        }
        if let Some(file) = &mut self.proc_net_snmp {
            let parsed = crate::common::nested_map_from_file(file).await?;
            let time = Instant::now();
            for statistic in &self.statistics {
                if let Some((pkey, lkey)) = statistic.keys() {
                    if let Some(inner) = parsed.get(pkey) {
                        if let Some(value) = inner.get(lkey) {
                            let _ = self.metrics().record_counter(statistic, time, *value);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn sample_netstat(&mut self) -> Result<(), std::io::Error> {
        if self.proc_net_netstat.is_none() {
            let file = File::open("/proc/net/netstat").await?;
            self.proc_net_netstat = Some(file);
        }
        if let Some(file) = &mut self.proc_net_netstat {
            let parsed = crate::common::nested_map_from_file(file).await?;
            let time = Instant::now();
            for statistic in &self.statistics {
                if let Some((pkey, lkey)) = statistic.keys() {
                    if let Some(inner) = parsed.get(pkey) {
                        if let Some(value) = inner.get(lkey) {
                            let _ = self.metrics().record_counter(statistic, time, *value);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
