// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::*;
use crate::config::Config;
use crate::samplers::Sampler;
use crate::stats::{record_counter, record_gauge, register_counter};
use failure::Error;
use logger::*;
use metrics::*;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::process;
use std::str;
use time;

pub struct Memcache<'a> {
    config: &'a Config,
    stream: TcpStream,
    initialized: bool,
    recorder: &'a Recorder<AtomicU32>,
}

impl<'a> Sampler<'a> for Memcache<'a> {
    fn new(
        config: &'a Config,
        recorder: &'a Recorder<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
        let endpoint = config.memcache().unwrap();
        let mut addrs = endpoint.to_socket_addrs().unwrap_or_else(|_| {
            println!("ERROR: endpoint address is malformed: {}", endpoint);
            std::process::exit(1);
        });
        let sock_addr = addrs.next().unwrap_or_else(|| {
            println!("ERROR: failed to resolve address: {}", endpoint);
            std::process::exit(1);
        });
        let stream = TcpStream::connect(sock_addr).unwrap_or_else(|e| {
            error!("Failed to connect to memcache: {}", e);
            process::exit(1);
        });
        Ok(Some(Box::new(Memcache {
            config,
            stream,
            initialized: false,
            recorder,
        })))
    }

    fn name(&self) -> String {
        "memcache".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        // gather current state
        trace!("sampling memcache");
        let time = time::precise_time_ns();
        self.stream.write_all(b"stats\r\n").unwrap();
        let mut buffer = [0_u8; 16355];
        loop {
            let length = self.stream.peek(&mut buffer).unwrap();
            if length > 0 {
                let stats = str::from_utf8(&buffer).unwrap().to_string();
                let lines: Vec<&str> = stats.split("\r\n").collect();
                if lines[lines.len() - 2] == "END" {
                    break;
                }
            }
        }
        let length = self.stream.read(&mut buffer).unwrap();
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
                                if !self.initialized {
                                    register_counter(
                                        self.recorder,
                                        name,
                                        BILLION,
                                        3,
                                        self.config.general().window(),
                                        PERCENTILES,
                                    );
                                }
                                if let Ok(value) = value.parse::<f64>().map(|v| v.floor() as u64) {
                                    record_counter(self.recorder, name.to_string(), time, value);
                                }
                            }
                            _ => {
                                if !self.initialized {
                                    self.recorder.add_channel(
                                        name.to_string(),
                                        Source::Gauge,
                                        None,
                                    );
                                    self.recorder.add_output(name.to_string(), Output::Counter);
                                }
                                if let Ok(value) = value.parse::<f64>().map(|v| v.floor() as u64) {
                                    record_gauge(self.recorder, name.to_string(), time, value);
                                }
                            }
                        }
                    }
                }
            }
            self.initialized = true;
        } else {
            error!("failed to get stats");
            std::process::exit(1);
        }
        Ok(())
    }

    fn register(&mut self) {
        // this is handled in-line
    }

    fn deregister(&mut self) {
        // this is not implemented
    }
}
