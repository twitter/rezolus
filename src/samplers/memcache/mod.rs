// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::*;
use crate::config::*;
use crate::samplers::{Common, Sampler};

use failure::Error;
use logger::*;
use metrics::*;
use time;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

use std::str;
use std::sync::Arc;

pub struct Memcache {
    address: SocketAddr,
    common: Common,
    stream: Option<TcpStream>,
}

impl Memcache {
    fn reconnect(&mut self) {
        if self.stream.is_none() {
            match TcpStream::connect(self.address) {
                Ok(stream) => {
                    info!("Successfully connected to memcache");
                    self.stream = Some(stream);
                }
                Err(e) => {
                    error!("Failed to connect to memcache: {}", e);
                }
            }
        }
    }
}

impl Sampler for Memcache {
    fn new(
        config: Arc<Config>,
        metrics: Metrics<AtomicU32>,
    ) -> Result<Option<Box<dyn Sampler>>, Error> {
        let endpoint = config.memcache().unwrap();
        let mut addrs = endpoint.to_socket_addrs().unwrap_or_else(|_| {
            println!("ERROR: endpoint address is malformed: {}", endpoint);
            std::process::exit(1);
        });
        let address = addrs.next().unwrap_or_else(|| {
            println!("ERROR: failed to resolve address: {}", endpoint);
            std::process::exit(1);
        });
        let mut sampler = Memcache { address, common: Common::new(config, metrics), stream: None};
        sampler.reconnect();
        Ok(Some(Box::new(sampler) as Box<dyn Sampler>))
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn name(&self) -> String {
        "memcache".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        // gather current state
        trace!("sampling memcache");
        let time = time::precise_time_ns();
        if let Some(ref mut stream) = self.stream {
            stream.write_all(b"stats\r\n").unwrap();
            let mut buffer = [0_u8; 16355];
            loop {
                let length = stream.peek(&mut buffer).unwrap();
                if length > 0 {
                    let stats = str::from_utf8(&buffer).unwrap().to_string();
                    let lines: Vec<&str> = stats.split("\r\n").collect();
                    if lines[lines.len() - 2] == "END" {
                        break;
                    }
                }
            }
            let length = stream.read(&mut buffer).unwrap();
            if length > 0 {
                let stats = str::from_utf8(&buffer).unwrap().to_string();
                let lines: Vec<&str> = stats.split("\r\n").collect();
                for line in lines {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(name) = parts.get(1) {
                        if let Some(value) = parts.get(2) {
                            match *name {
                                "data_read" | "data_written" | "cmd_total" | "conn_total"
                                | "conn_yield" | "hotkey_bw" | "hotkey_qps" => {
                                    if !self.common.initialized() {
                                        self.common.register_counter(name, BILLION, 3, PERCENTILES);
                                    }
                                    if let Ok(value) =
                                        value.parse::<f64>().map(|v| v.floor() as u64)
                                    {
                                        self.common.record_counter(name, time, value);
                                    }
                                }
                                _ => {
                                    if !self.common.initialized() {
                                        self.common.metrics().add_channel(
                                            name.to_string(),
                                            Source::Gauge,
                                            None,
                                        );
                                        self.common
                                            .metrics()
                                            .add_output(name.to_string(), Output::Counter);
                                    }
                                    if let Ok(value) =
                                        value.parse::<f64>().map(|v| v.floor() as u64)
                                    {
                                        self.common.record_gauge(name, time, value);
                                    }
                                }
                            }
                        }
                    }
                }
                self.common.set_initialized(true);
            } else {
                error!("failed to get stats. disconnect");
                self.stream = None;
            }
        } else {
            self.reconnect();
        }
        Ok(())
    }

    fn interval(&self) -> usize {
        self.common().config().interval()
    }

    fn register(&mut self) {
        // this is handled in-line
    }

    fn deregister(&mut self) {
        // this is not implemented
    }
}
