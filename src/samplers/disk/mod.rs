// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod device;
mod entry;

pub use self::device::Device;
pub use self::entry::Entry;
use crate::stats::record_counter;
use crate::stats::register_counter;
use failure::Error;

use crate::common::*;
use crate::config::Config;
use crate::samplers::Sampler;

use logger::*;
use metrics::*;
use regex::Regex;
use serde_derive::*;
use time;
use walkdir::WalkDir;

use std::collections::HashMap;

const REFRESH: u64 = 60_000_000_000;

pub struct Disk<'a> {
    config: &'a Config,
    devices: Vec<Device>,
    initialized: bool,
    last_refreshed: u64,
    recorder: &'a Recorder<AtomicU32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum Statistic {
    BandwidthRead,
    BandwidthWrite,
    OperationsRead,
    OperationsWrite,
}

impl std::fmt::Display for Statistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Statistic::BandwidthRead => write!(f, "disk/bandwidth/read"),
            Statistic::BandwidthWrite => write!(f, "disk/bandwidth/write"),
            Statistic::OperationsRead => write!(f, "disk/operations/read"),
            Statistic::OperationsWrite => write!(f, "disk/operations/write"),
        }
    }
}

impl<'a> Disk<'a> {
    /// send deltas to the stats library
    fn record(&self, time: u64, recorder: &Recorder<AtomicU32>, reading: Entry) {
        record_counter(
            recorder,
            Statistic::BandwidthRead,
            time,
            reading.read_bytes(),
        );
        record_counter(
            recorder,
            Statistic::BandwidthWrite,
            time,
            reading.write_bytes(),
        );
        record_counter(
            recorder,
            Statistic::OperationsRead,
            time,
            reading.read_ops(),
        );
        record_counter(
            recorder,
            Statistic::OperationsWrite,
            time,
            reading.write_ops(),
        );
    }

    /// identifies the set of all primary block `Device`s on the host
    fn get_devices(&self) -> Vec<Device> {
        let re = Regex::new(r"^[a-z]+$").unwrap();
        let mut result = Vec::new();
        for entry in WalkDir::new("/sys/class/block/").max_depth(1) {
            if let Ok(entry) = entry {
                if let Some(s) = entry.file_name().to_str() {
                    if s != "block" && re.is_match(s) {
                        trace!("Found block dev: {}", s);
                        result.push(Device::new(Some(s.to_owned())));
                    } else {
                        trace!("Ignore block dev: {}", s);
                    }
                }
            }
        }
        result
    }
}

impl<'a> Sampler<'a> for Disk<'a> {
    fn new(
        config: &'a Config,
        recorder: &'a Recorder<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error> {
        if config.disk().enabled() {
            Ok(Some(Box::new(Self {
                config,
                devices: Vec::new(),
                initialized: false,
                last_refreshed: 0,
                recorder,
            })))
        } else {
            Ok(None)
        }
    }

    fn name(&self) -> String {
        "disk".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        trace!("sample {}", self.name());
        let time = time::precise_time_ns();
        let mut current = HashMap::new();
        if (time - self.last_refreshed) >= REFRESH {
            self.devices = self.get_devices();
            self.last_refreshed = time;
        }
        for device in self.devices.clone() {
            let entry = Entry::for_device(&device);
            current.insert(device, entry);
        }
        if !self.initialized {
            self.register();
        }
        self.record(time, self.recorder, current.values().sum());
        Ok(())
    }

    fn register(&mut self) {
        trace!("register {}", self.name());
        if !self.initialized {
            self.devices = self.get_devices();
            self.last_refreshed = time::precise_time_ns();
            register_counter(
                self.recorder,
                Statistic::BandwidthRead,
                TRILLION,
                3,
                self.config.general().window(),
                PERCENTILES,
            );
            register_counter(
                self.recorder,
                Statistic::BandwidthWrite,
                TRILLION,
                3,
                self.config.general().window(),
                PERCENTILES,
            );
            register_counter(
                self.recorder,
                Statistic::OperationsRead,
                BILLION,
                3,
                self.config.general().window(),
                PERCENTILES,
            );
            register_counter(
                self.recorder,
                Statistic::OperationsWrite,
                BILLION,
                3,
                self.config.general().window(),
                PERCENTILES,
            );
            self.initialized = true;
        }
    }

    fn deregister(&mut self) {
        trace!("deregister {}", self.name());
        if self.initialized {
            self.recorder
                .delete_channel(Statistic::BandwidthRead.to_string());
            self.recorder
                .delete_channel(Statistic::BandwidthWrite.to_string());
            self.recorder
                .delete_channel(Statistic::OperationsRead.to_string());
            self.recorder
                .delete_channel(Statistic::OperationsWrite.to_string());
            self.initialized = false;
        }
    }
}
