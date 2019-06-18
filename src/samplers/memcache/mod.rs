// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::*;
use crate::config::Config;
use crate::sampler::{Sampler, SamplerError};
use crate::stats::{record_counter, record_gauge, register_counter};
use logger::*;
use metrics::*;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::process;
use std::str;
use time;

pub struct Memcache {
    _socket: SocketAddr,
    stream: TcpStream,
    initialized: bool,
}

impl Memcache {
    pub fn new(_config: &Config, socket: SocketAddr) -> Memcache {
        let stream = TcpStream::connect(socket).unwrap_or_else(|e| {
            error!("Failed to connect to memcache: {}", e);
            process::exit(1);
        });
        Memcache {
            _socket: socket,
            stream,
            initialized: false,
        }
    }
}

impl Sampler for Memcache {
    fn name(&self) -> String {
        "memcache".to_string()
    }

    fn sample(&mut self, recorder: &Recorder<u32>, config: &Config) -> Result<(), SamplerError> {
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
                                        recorder,
                                        name,
                                        BILLION,
                                        3,
                                        config.general().interval(),
                                        PERCENTILES,
                                    );
                                }
                                if let Ok(value) = value.parse::<f64>().map(|v| v.floor() as u64) {
                                    record_counter(recorder, name.to_string(), time, value);
                                }
                            }
                            _ => {
                                if !self.initialized {
                                    recorder.add_channel(name.to_string(), Source::Gauge, None);
                                    recorder.add_output(name.to_string(), Output::Counter);
                                }
                                if let Ok(value) = value.parse::<f64>().map(|v| v.floor() as u64) {
                                    record_gauge(recorder, name.to_string(), time, value);
                                }
                            }
                        }
                    }
                }
            }
            self.initialized = true;
        } else {
            error!("failed to get stats");
        }
        Ok(())
    }

    fn register(&mut self, _recorder: &Recorder<u32>, _config: &Config) {
        // this is handled in-line
    }

    fn deregister(&mut self, _recorder: &Recorder<u32>, _config: &Config) {
        // this is not implemented
    }
}
