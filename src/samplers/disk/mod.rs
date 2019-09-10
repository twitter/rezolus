// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod device;
mod entry;

pub use self::device::Device;
pub use self::entry::Entry;

use crate::common::*;
use crate::config::Config;
use crate::samplers::{Common, Sampler};

use failure::Error;
use logger::*;
use metrics::*;
use regex::Regex;
use serde_derive::*;
use time;
use walkdir::WalkDir;

use std::collections::HashMap;

const REFRESH: u64 = 60_000_000_000;

pub struct Disk<'a> {
    common: Common<'a>,
    devices: Vec<Device>,
    last_refreshed: u64,
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
    fn record(&self, time: u64, reading: Entry) {
        self.common
            .record_counter(&Statistic::BandwidthRead, time, reading.read_bytes());
        self.common
            .record_counter(&Statistic::BandwidthWrite, time, reading.write_bytes());
        self.common
            .record_counter(&Statistic::OperationsRead, time, reading.read_ops());
        self.common
            .record_counter(&Statistic::OperationsWrite, time, reading.write_ops());
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
                common: Common::new(config, recorder),
                devices: Vec::new(),
                last_refreshed: 0,
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
        self.register();
        self.record(time, current.values().sum());
        Ok(())
    }

    fn register(&mut self) {
        if !self.common.initialized() {
            trace!("register {}", self.name());
            self.devices = self.get_devices();
            self.last_refreshed = time::precise_time_ns();
            for statistic in &[Statistic::BandwidthRead, Statistic::BandwidthWrite] {
                self.common
                    .register_counter(statistic, TRILLION, 3, PERCENTILES);
            }
            for statistic in &[Statistic::OperationsRead, Statistic::OperationsWrite] {
                self.common
                    .register_counter(statistic, BILLION, 3, PERCENTILES);
            }
            self.common.set_initialized(true);
        }
    }

    fn deregister(&mut self) {
        if self.common.initialized() {
            trace!("deregister {}", self.name());
            for statistic in &[
                Statistic::BandwidthRead,
                Statistic::BandwidthWrite,
                Statistic::OperationsRead,
                Statistic::OperationsWrite,
            ] {
                self.common.delete_channel(statistic);
            }
            self.common.set_initialized(false);
        }
    }
}
