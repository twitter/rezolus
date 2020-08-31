// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

use async_trait::async_trait;
use rustcommon_metrics::*;

use crate::config::*;
use crate::samplers::Common;
use crate::Sampler;

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

    fn new(common: Common) -> Result<Self, failure::Error> {
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
        let ret = Self {
            address,
            common,
            stream: None,
        };
        Ok(ret)
    }

    fn spawn(common: Common) {
        if let Ok(mut sampler) = Self::new(common.clone()) {
            common.handle.spawn(async move {
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
                let time = time::precise_time_ns();
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
                                            .add_output(&statistic, Output::Reading);
                                        self.common()
                                            .metrics()
                                            .record_counter(&statistic, time, value);
                                        for percentile in self.sampler_config().percentiles() {
                                            self.common().metrics().add_output(
                                                &statistic,
                                                Output::Percentile(*percentile),
                                            );
                                        }
                                    }
                                }
                                _ => {
                                    if let Ok(value) =
                                        value.parse::<f64>().map(|v| v.floor() as u64)
                                    {
                                        let statistic = MemcacheStatistic::new((*name).to_string());
                                        self.common().metrics().register(&statistic, None);
                                        self.common()
                                            .metrics()
                                            .add_output(&statistic, Output::Reading);
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
            }
        } else {
            if let Ok(stream) = TcpStream::connect(self.address) {
                let _ = stream.set_nonblocking(true);
                self.stream = Some(stream);
            } else {
                error!("error connecting to memcache");
            }
        }

        Ok(())
    }
}
