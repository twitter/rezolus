// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub(crate) mod interface;
pub(crate) mod protocol;
pub(crate) mod statistics;

pub use self::interface::*;

use self::statistics::InterfaceStatistic;
use crate::common::*;
use crate::config::Config;
use crate::samplers::{Common, Sampler};

use failure::Error;
use logger::*;
use metrics::*;
use time;
use walkdir;

use std::collections::HashSet;

const REFRESH: u64 = 60_000_000_000;

pub struct Network<'a> {
    common: Common<'a>,
    interfaces: HashSet<Interface>,
    last_refreshed: u64,
}

impl<'a> Network<'a> {
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

impl<'a> Sampler<'a> for Network<'a> {
    fn new(
        config: &'a Config,
        recorder: &'a Recorder<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
        if config.network().enabled() {
            Ok(Some(Box::new(Self {
                common: Common::new(config, recorder),
                interfaces: HashSet::new(),
                last_refreshed: 0,
            })))
        } else {
            Ok(None)
        }
    }

    fn name(&self) -> String {
        "network".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        trace!("sample {}", self.name());
        let time = time::precise_time_ns();
        if (time - self.last_refreshed) >= REFRESH {
            self.interfaces = self.get_interfaces();
            self.last_refreshed = time;
        }
        self.register();

        // interface statistics
        for statistic in self.common.config().network().interface_statistics() {
            let sum: u64 = self
                .interfaces
                .iter()
                .map(|i| i.get_statistic(&statistic).unwrap_or(0))
                .sum();
            self.common.record_counter(&statistic, time, sum);
        }

        // protocol statistics
        if let Ok(protocol) = protocol::Protocol::new() {
            for statistic in self.common.config().network().protocol_statistics() {
                let value = *protocol.get(statistic).unwrap_or(&0);
                self.common.record_counter(statistic, time, value);
            }
        }

        Ok(())
    }

    fn register(&mut self) {
        if !self.common.initialized() {
            trace!("register {}", self.name());
            self.interfaces = self.get_interfaces();
            self.last_refreshed = time::precise_time_ns();
            let mut total_bandwidth_bytes = 0;
            for interface in self.interfaces.iter() {
                if interface.name().is_some() {
                    total_bandwidth_bytes += interface.bandwidth_bytes().unwrap_or(0);
                }
            }
            for statistic in self.common.config().network().interface_statistics() {
                let max = match statistic {
                    InterfaceStatistic::RxBytes | InterfaceStatistic::TxBytes => {
                        2 * total_bandwidth_bytes
                    }
                    _ => (2 * total_bandwidth_bytes / 64),
                };
                self.common
                    .register_counter(&statistic, max, 3, PERCENTILES);
            }
            for statistic in self.common.config().network().protocol_statistics() {
                let max = 2 * total_bandwidth_bytes / 64;
                self.common.register_counter(statistic, max, 3, PERCENTILES);
            }
            self.common.set_initialized(true);
        }
    }

    fn deregister(&mut self) {
        if self.common.initialized() {
            trace!("deregister {}", self.name());
            for statistic in self.common.config().network().interface_statistics() {
                self.common.delete_channel(statistic);
            }
            for statistic in self.common.config().network().protocol_statistics() {
                self.common.delete_channel(statistic);
            }
            self.common.set_initialized(false);
        }
    }
}
