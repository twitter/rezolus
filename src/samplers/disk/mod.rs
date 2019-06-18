// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod device;
mod entry;

pub use self::device::Device;
pub use self::entry::Entry;
use crate::stats::record_counter;
use crate::stats::register_counter;

use crate::common::*;
use crate::config::Config;
use crate::sampler::{Sampler, SamplerError};

use logger::*;
use metrics::*;
use regex::Regex;
use time;
use walkdir::WalkDir;

use std::collections::HashMap;

const REFRESH: u64 = 60_000_000_000;

pub struct Disk {
    devices: Vec<Device>,
    initialized: bool,
    last_refreshed: u64,
}

impl Disk {
    /// create a new `Sampler` for Disk subsystem
    pub fn new(_config: &Config) -> Self {
        Self {
            devices: Vec::new(),
            initialized: false,
            last_refreshed: 0,
        }
    }

    /// send deltas to the stats library
    fn record(&self, time: u64, recorder: &Recorder<u32>, reading: Entry) {
        record_counter(
            recorder,
            "disk/bandwidth/read".to_string(),
            time,
            reading.read_bytes(),
        );
        record_counter(
            recorder,
            "disk/bandwidth/write".to_string(),
            time,
            reading.write_bytes(),
        );
        record_counter(
            recorder,
            "disk/operations/read".to_string(),
            time,
            reading.read_ops(),
        );
        record_counter(
            recorder,
            "disk/operations/write".to_string(),
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

impl Sampler for Disk {
    fn name(&self) -> String {
        "disk".to_string()
    }

    fn sample(&mut self, recorder: &Recorder<u32>, config: &Config) -> Result<(), SamplerError> {
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
            self.register(recorder, config);
        }
        self.record(time, recorder, current.values().sum());
        Ok(())
    }

    fn register(&mut self, recorder: &Recorder<u32>, config: &Config) {
        trace!("register {}", self.name());
        if !self.initialized {
            self.devices = self.get_devices();
            self.last_refreshed = time::precise_time_ns();
            register_counter(
                recorder,
                "disk/bandwidth/read",
                TRILLION,
                3,
                config.general().interval(),
                PERCENTILES,
            );
            register_counter(
                recorder,
                "disk/bandwidth/write",
                TRILLION,
                3,
                config.general().interval(),
                PERCENTILES,
            );
            register_counter(
                recorder,
                "disk/operations/read",
                BILLION,
                3,
                config.general().interval(),
                PERCENTILES,
            );
            register_counter(
                recorder,
                "disk/operations/read",
                BILLION,
                3,
                config.general().interval(),
                PERCENTILES,
            );
            self.initialized = true;
        }
    }

    fn deregister(&mut self, recorder: &Recorder<u32>, _config: &Config) {
        trace!("deregister {}", self.name());
        if self.initialized {
            recorder.delete_channel("disk/bandwidth/read".to_string());
            recorder.delete_channel("disk/bandwidth/write".to_string());
            recorder.delete_channel("disk/operations/read".to_string());
            recorder.delete_channel("disk/operations/read".to_string());
            self.initialized = false;
        }
    }
}
