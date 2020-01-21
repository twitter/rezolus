// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::bpf::*;
use crate::config::{Config, SamplerConfig};
use crate::samplers::{Common, Sampler};
use tokio::runtime::Handle;

use async_trait::async_trait;
#[cfg(feature = "ebpf")]
use bcc;
use metrics::*;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

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
    fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Result<Self, failure::Error> {
        #[cfg(feature = "ebpf")]
        let bpf = if config.samplers().tcp().ebpf() {
            debug!("initializing ebpf");
            // load the code and compile
            let code = include_str!("bpf.c");
            let mut bpf = bcc::core::BPF::new(code)?;

            // load + attach kprobes!
            let tcp_v4_connect = bpf.load_kprobe("trace_connect")?;
            let tcp_v6_connect = bpf.load_kprobe("trace_connect")?;
            let tcp_rcv_state_process = bpf.load_kprobe("trace_tcp_rcv_state_process")?;
            bpf.attach_kprobe("tcp_v4_connect", tcp_v4_connect)?;
            bpf.attach_kprobe("tcp_v6_connect", tcp_v6_connect)?;
            bpf.attach_kprobe("tcp_rcv_state_process", tcp_rcv_state_process)?;

            Some(Arc::new(Mutex::new(BPF { inner: bpf })))
        } else {
            None
        };

        #[cfg(not(feature = "ebpf"))]
        let bpf = None;

        Ok(Self {
            bpf,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
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
        self.common.config().samplers().tcp()
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

        // sample ebpf
        #[cfg(feature = "ebpf")]
        {
            if self.bpf_last.lock().unwrap().elapsed() >= self.general_config().window() {
                if let Some(ref bpf) = self.bpf {
                    let bpf = bpf.lock().unwrap();
                    let time = time::precise_time_ns();
                    for statistic in self.sampler_config().statistics() {
                        if let Some(table) = statistic.ebpf_table() {
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
        }

        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
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
