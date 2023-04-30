// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

use async_trait::async_trait;

use crate::config::*;
use crate::samplers::Common;
use crate::*;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

pub struct Memcache {
    address: SocketAddr,
    common: Common,
    stream: Option<TcpStream>,
}

#[async_trait]
impl Sampler for Memcache {
    type Statistic = MemcacheStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        if !common.config.samplers().memcache().enabled() {
            return Ok(Self {
                address: "localhost:11211".to_socket_addrs().unwrap().next().unwrap(),
                common,
                stream: None,
            });
        }
        if common.config.samplers().memcache().endpoint().is_none() {
            return Err(format_err!("no memcache endpoint configured"));
        }
        let endpoint = common.config.samplers().memcache().endpoint().unwrap();
        let mut addrs = endpoint.to_socket_addrs().unwrap_or_else(|_| {
            fatal!("ERROR: endpoint address is malformed: {}", endpoint);
        });
        let address = addrs.next().unwrap_or_else(|| {
            fatal!("ERROR: failed to resolve address: {}", endpoint);
        });
        let sampler = Self {
            address,
            common,
            stream: None,
        };
        if sampler.sampler_config().enabled() {
            sampler.register();
        }
        Ok(sampler)
    }

    fn spawn(common: Common) {
        if common.config().samplers().memcache().enabled() {
            if let Ok(mut sampler) = Self::new(common.clone()) {
                common.runtime().spawn(async move {
                    loop {
                        let _ = sampler.sample().await;
                    }
                });
            } else if !common.config.fault_tolerant() {
                fatal!("failed to initialize memcache sampler");
            } else {
                error!("failed to initialize memcache sampler");
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

        if let Some(ref mut stream) = self.stream {
            if stream.write_all(b"stats\r\n").is_ok() {
                loop {
                    let mut buffer = [0_u8; 65536];
                    if let Ok(length) = stream.peek(&mut buffer) {
                        if length == 0 {
                            error!("zero length read. disconnect");
                            self.stream = None;
                            return Ok(());
                        }
                        let stats = std::str::from_utf8(&buffer).unwrap().to_string();
                        let lines: Vec<&str> = stats.split("\r\n").collect();
                        if lines.len() >= 2 && lines[lines.len() - 2] == "END" {
                            break;
                        }
                    }
                }
                let mut buffer = [0_u8; 65536];
                let _ = stream.read(&mut buffer);
                let time = Instant::now();
                let stats = std::str::from_utf8(&buffer).unwrap().to_string();
                let lines: Vec<&str> = stats.split("\r\n").collect();
                for line in lines {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(name) = parts.get(1) {
                        if let Some(Ok(value)) = parts
                            .get(2)
                            .map(|v| v.parse::<f64>().map(|v| v.floor() as u64))
                        {
                            let statistic = MemcacheStatistic::new((*name).to_string());

                            // all statistics will be registered and have the
                            // current value as an output
                            self.common().metrics().register(&statistic);
                            self.common()
                                .metrics()
                                .add_output(&statistic, Output::Reading);

                            // for some statistics, we will export summary stats
                            if statistic.summary_type().is_some() {
                                self.common()
                                    .metrics()
                                    .add_summary(&statistic, Summary::stream(self.samples()));
                                for percentile in self.sampler_config().percentiles() {
                                    self.common()
                                        .metrics()
                                        .add_output(&statistic, Output::Percentile(*percentile));
                                }
                            }

                            // all statistics should have their current value
                            // recorded
                            match statistic.source() {
                                Source::Counter => {
                                    let _ = self
                                        .common()
                                        .metrics()
                                        .record_counter(&statistic, time, value);
                                }
                                Source::Gauge => {
                                    let _ = self
                                        .common()
                                        .metrics()
                                        .record_gauge(&statistic, time, value);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            } else {
                // write failed, so we should disconnect
                self.stream = None;
            }
        } else if let Ok(stream) =
            TcpStream::connect_timeout(&self.address, std::time::Duration::from_millis(100))
        {
            let _ = stream.set_nonblocking(true);
            self.stream = Some(stream);
        } else {
            // delay here if we hit a connect failure
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            error!("error connecting to memcache");
        }

        Ok(())
    }

    fn config(common: &Common) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        common.config().samplers().memcache()
    }
}
