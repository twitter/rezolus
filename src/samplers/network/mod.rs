// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use rustcommon_metrics::*;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::common::bpf::*;
use crate::common::*;
use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::Sampler;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Network {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
}

#[async_trait]
impl Sampler for Network {
    type Statistic = NetworkStatistic;

    fn new(common: Common) -> Result<Self, failure::Error> {
        let fault_tolerant = common.config.general().fault_tolerant();

        #[allow(unused_mut)]
        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
        };

        if let Err(e) = sampler.initialize_bpf() {
            if !fault_tolerant {
                return Err(e);
            }
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
            fatal!("failed to initialize network sampler");
        } else {
            error!("failed to initialize network sampler");
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().network()
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

        self.map_result(self.sample_proc_net_dev().await)?;
        #[cfg(feature = "bpf")]
        self.map_result(self.sample_bpf())?;

        Ok(())
    }

    fn summary(&self, statistic: &Self::Statistic) -> Option<Summary> {
        let precision = if statistic.bpf_table().is_some() {
            2
        } else {
            3
        };

        let max = if statistic.bpf_table().is_some() {
            SECOND
        } else {
            TERABIT
        };

        Some(Summary::histogram(
            max,
            precision,
            Some(self.general_config().window()),
        ))
    }
}

impl Network {
    // checks that bpf is enabled in config and one or more bpf stats enabled
    #[cfg(feature = "bpf")]
    fn bpf_enabled(&self) -> bool {
        if self.sampler_config().bpf() {
            for statistic in self.sampler_config().statistics() {
                if statistic.bpf_table().is_some() {
                    return true;
                }
            }
        }
        false
    }

    fn initialize_bpf(&mut self) -> Result<(), failure::Error> {
        #[cfg(feature = "bpf")]
        {
            if self.enabled() && self.bpf_enabled() {
                debug!("initializing bpf");
                // load the code and compile
                let code = include_str!("bpf.c");
                let mut bpf = bcc::BPF::new(code)?;

                bcc::Tracepoint::new()
                    .handler("trace_transmit")
                    .subsystem("net")
                    .tracepoint("net_dev_queue")
                    .attach(&mut bpf)?;
                bcc::Tracepoint::new()
                    .handler("trace_receive")
                    .subsystem("net")
                    .tracepoint("netif_rx")
                    .attach(&mut bpf)?;

                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
            }
        }

        Ok(())
    }

    async fn sample_proc_net_dev(&self) -> Result<(), std::io::Error> {
        // sample /proc/net/dev
        let file = File::open("/proc/net/dev").await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut result = HashMap::new();

        while let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() && parts[1].parse::<u64>().is_ok() {
                for statistic in self.sampler_config().statistics() {
                    if let Some(field) = statistic.field_number() {
                        if !result.contains_key(statistic) {
                            result.insert(*statistic, 0);
                        }
                        let current = result.get_mut(statistic).unwrap();
                        *current += parts
                            .get(field)
                            .map(|v| v.parse().unwrap_or(0))
                            .unwrap_or(0);
                    }
                }
            }
        }

        let time = time::precise_time_ns();
        for (stat, value) in result {
            self.metrics().record_counter(&stat, time, value);
        }
        Ok(())
    }

    #[cfg(feature = "bpf")]
    fn sample_bpf(&self) -> Result<(), std::io::Error> {
        if self.bpf_last.lock().unwrap().elapsed() >= self.general_config().window() {
            let time = time::precise_time_ns();
            if let Some(ref bpf) = self.bpf {
                let bpf = bpf.lock().unwrap();
                for statistic in self.sampler_config().statistics() {
                    if let Some(table) = statistic.bpf_table() {
                        let mut table = (*bpf).inner.table(table);

                        for (&value, &count) in &map_from_table(&mut table) {
                            if count > 0 {
                                self.metrics()
                                    .record_distribution(statistic, time, value, count);
                            }
                        }
                    }
                }
            }
            *self.bpf_last.lock().unwrap() = Instant::now();
        }
        Ok(())
    }
}
