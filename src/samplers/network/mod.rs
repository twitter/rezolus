// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::bpf::*;
use crate::common::*;
use crate::config::{Config, SamplerConfig};
use crate::samplers::Common;
use std::sync::Mutex;
use tokio::runtime::Handle;

use crate::Sampler;
use async_trait::async_trait;
#[cfg(feature = "ebpf")]
use bcc;
use metrics::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

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

    fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Result<Self, failure::Error> {
        let fault_tolerant = config.general().fault_tolerant();

        #[allow(unused_mut)]
        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common: Common::new(config, metrics),
        };

        if let Err(e) = sampler.initialize_ebpf() {
            if !fault_tolerant {
                return Err(e);
            }
        }

        Ok(sampler)
    }

    fn spawn(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>, handle: &Handle) {
        if let Ok(mut sampler) = Self::new(config.clone(), metrics) {
            handle.spawn(async move {
                loop {
                    let _ = sampler.sample().await;
                }
            });
        } else if !config.fault_tolerant() {
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

        // sample /proc/net/dev
        let file = File::open("/proc/net/dev").await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut result = HashMap::new();

        while let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts[1].parse::<u64>().is_ok() {
                for statistic in self.sampler_config().statistics() {
                    if let Some(field) = statistic.field_number() {
                        if !result.contains_key(statistic) {
                            result.insert(*statistic, 0);
                        }
                        let current = result.get_mut(statistic).unwrap();
                        *current += parts[field].parse().unwrap_or(0);
                    }
                }
            }
        }

        let time = time::precise_time_ns();
        for (stat, value) in result {
            self.metrics().record_counter(&stat, time, value);
        }

        // sample ebpf
        #[cfg(feature = "ebpf")]
        {
            if self.bpf_last.lock().unwrap().elapsed() >= self.general_config().window() {
                if let Some(ref bpf) = self.bpf {
                    let bpf = bpf.lock().unwrap();
                    for statistic in self.sampler_config().statistics() {
                        if let Some(table) = statistic.ebpf_table() {
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
        }

        Ok(())
    }

    fn summary(&self, statistic: &Self::Statistic) -> Option<Summary> {
        let precision = if statistic.ebpf_table().is_some() {
            2
        } else {
            3
        };

        let max = if statistic.ebpf_table().is_some() {
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
    // checks that ebpf is enabled in config and one or more ebpf stats enabled
    #[cfg(feature = "ebpf")]
    fn ebpf_enabled(&self) -> bool {
        if self.sampler_config().ebpf() {
            for statistic in self.sampler_config().statistics() {
                if statistic.ebpf_table().is_some() {
                    return true;
                }
            }
        }
        false
    }

    fn initialize_ebpf(&self) -> Result<(), failure::Error> {
        #[cfg(feature = "ebpf")]
        {
            if sampler.ebpf_enabled() {
                debug!("initializing ebpf");
                // load the code and compile
                let code = include_str!("bpf.c");
                let mut bpf = bcc::core::BPF::new(code)?;

                // load + attach kprobes!
                let trace_transmit = bpf.load_tracepoint("trace_transmit")?;
                bpf.attach_tracepoint("net", "net_dev_queue", trace_transmit)?;
                let trace_receive = bpf.load_tracepoint("trace_receive")?;
                bpf.attach_tracepoint("net", "netif_rx", trace_receive)?;

                sampler.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
            }
        }

        Ok(())
    }
}
