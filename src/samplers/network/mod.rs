// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod interface;
mod protocol;

pub use self::interface::Interface;
use crate::stats::{record_counter, register_counter};

use crate::common::*;
use crate::config::Config;
use crate::sampler::{Sampler, SamplerError};

use logger::*;
use metrics::*;
use time;
use walkdir;

use std::collections::HashSet;

const REFRESH: u64 = 60_000_000_000;

pub struct Network {
    initialized: bool,
    interfaces: HashSet<Interface>,
    last_refreshed: u64,
}

impl Network {
    pub fn new(_config: &Config) -> Self {
        Self {
            initialized: false,
            interfaces: HashSet::new(),
            last_refreshed: 0,
        }
    }

    fn get_interfaces(&self) -> HashSet<Interface> {
        let mut interfaces = HashSet::default();
        for entry in walkdir::WalkDir::new("/sys/class/net/")
            .max_depth(1)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if let Some(name) = entry.file_name().to_str().to_owned() {
                trace!("Discovered NIC: {}", name);
                if !name.starts_with("eth") && !name.starts_with("en") && !name.starts_with("em") {
                    trace!("Ignore NIC: bad prefix: {}", name);
                    continue;
                }
                if !net::is_nic_active(name) {
                    trace!("Ignore NIC: inactive: {}", name);
                    continue;
                }
                if let Ok(speed) = file::file_as_u64(format!("/sys/class/net/{}/speed", name)) {
                    trace!("Monitoring NIC: {} speed: {} mbps", name, speed);
                    let bytes_secondly = (speed * 1_000_000) / 8;
                    interfaces.insert(Interface::new(Some(name.to_owned()), Some(bytes_secondly)));
                } else {
                    trace!("Ignore NIC: unknown speed: {}", name);
                }
            }
        }
        interfaces
    }
}

impl Sampler for Network {
    fn name(&self) -> String {
        "network".to_string()
    }

    fn sample(&mut self, recorder: &Recorder<u32>, config: &Config) -> Result<(), SamplerError> {
        trace!("sample {}", self.name());
        let time = time::precise_time_ns();
        if (time - self.last_refreshed) >= REFRESH {
            self.interfaces = self.get_interfaces();
            self.last_refreshed = time;
        }
        if !self.initialized {
            self.register(recorder, config);
        }

        // interface statistics
        for statistic in config.network().interface_statistics() {
            if let Ok(statistic) = self::interface::Statistic::from_str(&statistic) {
                let sum: u64 = self
                    .interfaces
                    .iter()
                    .map(|i| i.get_statistic(&statistic).unwrap_or(0))
                    .sum();
                record_counter(recorder, statistic.label(), time, sum);
            }
        }

        // protocol statistics
        if let Ok(protocol) = protocol::Protocol::new() {
            for statistic in config.network().protocol_statistics() {
                if let Ok(statistic) = self::protocol::Statistic::from_str(&statistic) {
                    let value = *protocol.get(&statistic).unwrap_or(&0);
                    record_counter(recorder, statistic.label(), time, value);
                }
            }
        }

        Ok(())
    }

    fn register(&mut self, recorder: &Recorder<u32>, config: &Config) {
        trace!("register {}", self.name());
        if !self.initialized {
            self.interfaces = self.get_interfaces();
            self.last_refreshed = time::precise_time_ns();
            let mut total_bandwidth_bytes = 0;
            for interface in self.interfaces.iter() {
                if interface.name().is_some() {
                    total_bandwidth_bytes += interface.bandwidth_bytes().unwrap_or(0);
                }
            }
            for statistic in config.network().interface_statistics() {
                if let Ok(statistic) = interface::Statistic::from_str(statistic) {
                    let max = match statistic {
                        interface::Statistic::RxBytes | interface::Statistic::TxBytes => {
                            2 * total_bandwidth_bytes
                        }
                        _ => (2 * total_bandwidth_bytes / 64),
                    };
                    register_counter(
                        recorder,
                        statistic.label(),
                        max,
                        3,
                        config.general().interval(),
                        PERCENTILES,
                    );
                }
            }
            for statistic in config.network().protocol_statistics() {
                if let Ok(statistic) = protocol::Statistic::from_str(statistic) {
                    let max = 2 * total_bandwidth_bytes / 64;
                    register_counter(
                        recorder,
                        statistic.label(),
                        max,
                        3,
                        config.general().interval(),
                        PERCENTILES,
                    );
                }
            }
            self.initialized = true;
        }
    }

    fn deregister(&mut self, recorder: &Recorder<u32>, config: &Config) {
        trace!("deregister {}", self.name());
        if self.initialized {
            for statistic in config.network().interface_statistics() {
                if let Ok(statistic) = interface::Statistic::from_str(statistic) {
                    recorder.delete_channel(statistic.label().to_string());
                }
            }
            for statistic in config.network().protocol_statistics() {
                if let Ok(statistic) = protocol::Statistic::from_str(statistic) {
                    recorder.delete_channel(statistic.label().to_string())
                }
            }
            self.initialized = false;
        }
    }
}
