// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;
use crate::samplers::Common;
use crate::Sampler;
use async_trait::async_trait;
use atomics::AtomicU32;
use metrics::*;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::runtime::Handle;

mod config;
pub use config::*;

mod stat;
pub use stat::*;

pub struct Memcache {
    address: SocketAddr,
    common: Common,
    stream: Option<TcpStream>,
}

impl Memcache {
    fn reconnect(&mut self) {
        if self.stream.is_none() {
            match std::net::TcpStream::connect(self.address) {
                Ok(stream) => match TcpStream::from_std(stream) {
                    Ok(stream) => {
                        info!("Connected to memcache");
                        self.stream = Some(stream)
                    }
                    Err(e) => {
                        error!("Failed to create tokio TcpStream: {}", e);
                    }
                },
                Err(e) => {
                    error!("Failed to connect to memcache: {}", e);
                }
            }
        }
    }
}

#[async_trait]
impl Sampler for Memcache {
    type Statistic = MemcacheStatistic;

    fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Result<Self, failure::Error> {
        if config.samplers().memcache().endpoint().is_none() {
            return Err(format_err!("no memcache endpoint configured"));
        }
        let endpoint = config.samplers().memcache().endpoint().unwrap();
        let mut addrs = endpoint.to_socket_addrs().unwrap_or_else(|_| {
            fatal!("ERROR: endpoint address is malformed: {}", endpoint);
        });
        let address = addrs.next().unwrap_or_else(|| {
            fatal!("ERROR: failed to resolve address: {}", endpoint);
        });
        let mut ret = Self {
            address,
            common: Common::new(config, metrics),
            stream: None,
        };
        ret.reconnect();
        Ok(ret)
    }

    fn spawn(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>, handle: &Handle) {
        if let Ok(mut sampler) = Self::new(config.clone(), metrics) {
            handle.spawn(async move {
                loop {
                    let _ = sampler.sample().await;
                }
            });
        } else if !config.fault_tolerant() {
            fatal!("failed to initialize memcache sampler");
        } else {
            error!("failed to initialize memcache sampler");
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().memcache()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }

        debug!("sampling");
        if let Some(ref mut stream) = self.stream {
            stream.write_all(b"stats\r\n").await?;
            let mut buffer = [0_u8; 16355];
            loop {
                let length = stream.peek(&mut buffer).await?;
                if length > 0 {
                    let stats = std::str::from_utf8(&buffer).unwrap().to_string();
                    let lines: Vec<&str> = stats.split("\r\n").collect();
                    if lines[lines.len() - 2] == "END" {
                        break;
                    }
                }
            }
            let time = time::precise_time_ns();
            let length = stream.read(&mut buffer).await?;
            if length > 0 {
                let stats = std::str::from_utf8(&buffer).unwrap().to_string();
                let lines: Vec<&str> = stats.split("\r\n").collect();
                for line in lines {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(name) = parts.get(1) {
                        if let Some(value) = parts.get(2) {
                            match *name {
                                "data_read" | "data_written" | "cmd_total" | "conn_total"
                                | "conn_yield" | "hotkey_bw" | "hotkey_qps" => {
                                    if let Ok(value) =
                                        value.parse::<f64>().map(|v| v.floor() as u64)
                                    {
                                        let statistic = MemcacheStatistic::new((*name).to_string());
                                        // these select metrics get histogram summaries and
                                        // percentile output
                                        self.common().metrics().register(
                                            &statistic,
                                            Some(Summary::histogram(
                                                1_000_000_000,
                                                3,
                                                Some(self.general_config().window()),
                                            )),
                                        );
                                        self.common()
                                            .metrics()
                                            .record_counter(&statistic, time, value);
                                        if self.summary(&statistic).is_some() {
                                            for percentile in self.sampler_config().percentiles() {
                                                self.common().metrics().register_output(
                                                    &statistic,
                                                    Output::Percentile(*percentile),
                                                );
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    if let Ok(value) =
                                        value.parse::<f64>().map(|v| v.floor() as u64)
                                    {
                                        let statistic = MemcacheStatistic::new((*name).to_string());
                                        // gauge type is used to pass-through raw metrics
                                        self.common()
                                            .metrics()
                                            .record_gauge(&statistic, time, value);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                error!("failed to get stats. disconnect");
                self.stream = None;
            }
        } else {
            self.reconnect();
        }

        Ok(())
    }
}
