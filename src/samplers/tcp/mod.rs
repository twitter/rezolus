// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use rustcommon_metrics::*;

use crate::common::bpf::*;
use crate::config::SamplerConfig;
use crate::samplers::{Common, Sampler};

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Tcp {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
}

#[async_trait]
impl Sampler for Tcp {
    type Statistic = TcpStatistic;
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
            fatal!("failed to initialize tcp sampler");
        } else {
            error!("failed to initialize tcp sampler");
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().tcp()
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

        // sample bpf
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
            1_000_000
        } else {
            1_000_000_000_000
        };

        Some(Summary::histogram(
            max,
            precision,
            Some(self.general_config().window()),
        ))
    }
}

impl Tcp {
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
                let mut bpf = bcc::core::BPF::new(code)?;

                // load + attach kprobes!
                bcc::core::kprobe::Kprobe::new()
                    .name("trace_connect")
                    .function("tcp_v4_connect")
                    .attach(&mut bpf)?;
                bcc::core::kprobe::Kprobe::new()
                    .name("trace_connect")
                    .function("tcp_v6_connect")
                    .attach(&mut bpf)?;
                bcc::core::kprobe::Kprobe::new()
                    .name("trace_tcp_rcv_state_process")
                    .function("tcp_rcv_state_process")
                    .attach(&mut bpf)?;

                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })))
            }
        }

        Ok(())
    }

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

    #[cfg(feature = "bpf")]
    fn sample_bpf(&self) -> Result<(), std::io::Error> {
        if self.bpf_last.lock().unwrap().elapsed() >= self.general_config().window() {
            if let Some(ref bpf) = self.bpf {
                let bpf = bpf.lock().unwrap();
                let time = time::precise_time_ns();
                for statistic in self.sampler_config().statistics() {
                    if let Some(table) = statistic.bpf_table() {
                        let mut table = (*bpf).inner.table(table);

                        for (&value, &count) in &map_from_table(&mut table) {
                            if count > 0 {
                                self.metrics().record_distribution(
                                    statistic,
                                    time,
                                    value * 1000,
                                    count,
                                );
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
