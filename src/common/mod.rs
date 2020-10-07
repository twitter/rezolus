// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
use std::io::BufRead;
use std::io::SeekFrom;

use dashmap::DashMap;
use rustcommon_atomics::AtomicU64;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

pub mod bpf;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

pub const SECOND: u64 = 1_000 * MILLISECOND;
pub const MILLISECOND: u64 = 1_000 * MICROSECOND;
pub const MICROSECOND: u64 = 1_000 * NANOSECOND;
pub const NANOSECOND: u64 = 1;

pub struct HardwareInfo {
    numa_mapping: DashMap<u64, u64>,
}

impl HardwareInfo {
    pub fn new() -> Self {
        let numa_mapping = DashMap::new();
        let mut node = 0;
        loop {
            let path = format!("/sys/devices/system/node/node{}/cpulist", node);
            if let Ok(f) = std::fs::File::open(path) {
                let mut reader = std::io::BufReader::new(f);
                let mut line = String::new();
                if reader.read_line(&mut line).is_ok() {
                    let ranges: Vec<&str> = line.trim().split(',').collect();
                    for range in ranges {
                        let parts: Vec<&str> = range.split('-').collect();
                        if parts.len() == 1 {
                            if let Ok(id) = parts[0].parse() {
                                numa_mapping.insert(id, node);
                            }
                        } else if parts.len() == 2 {
                            if let Ok(start) = parts[0].parse() {
                                if let Ok(stop) = parts[1].parse() {
                                    for id in start..=stop {
                                        numa_mapping.insert(id, node);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                break;
            }
            node += 1;
        }
        Self { numa_mapping }
    }

    pub fn get_numa(&self, core: u64) -> Option<u64> {
        self.numa_mapping.get(&core).map(|v| *v.value())
    }
}

/// helper function to discover the number of hardware threads
pub fn hardware_threads() -> Result<u64, ()> {
    let path = "/sys/devices/system/cpu/present";
    let f =
        std::fs::File::open(path).map_err(|e| debug!("failed to open file ({:?}): {}", path, e))?;
    let mut f = std::io::BufReader::new(f);

    let mut line = String::new();
    f.read_line(&mut line)
        .map_err(|_| debug!("failed to read line"))?;
    let line = line.trim();
    let a: Vec<&str> = line.split('-').collect();
    a.last()
        .unwrap_or(&"0")
        .parse::<u64>()
        .map_err(|e| debug!("could not parse num cpus from file ({:?}): {}", path, e))
        .map(|i| i + 1)
}

/// helper function to create a nested map from files with the form of
/// pkey1 lkey1 lkey2 ... lkeyN
/// pkey1 value1 value2 ... valueN
/// pkey2 ...
pub async fn nested_map_from_file(
    file: &mut File,
) -> Result<HashMap<String, HashMap<String, u64>>, std::io::Error> {
    file.seek(SeekFrom::Start(0)).await?;
    let mut ret = HashMap::<String, HashMap<String, u64>>::new();
    let mut reader = BufReader::new(file);
    let mut keys = String::new();
    let mut values = String::new();
    while reader.read_line(&mut keys).await? > 0 {
        if reader.read_line(&mut values).await? > 0 {
            let mut keys_split = keys.trim().split_whitespace();
            let mut values_split = values.trim().split_whitespace();

            if let Some(pkey) = keys_split.next() {
                let _ = values_split.next();
                if !ret.contains_key(pkey) {
                    ret.insert(pkey.to_string(), Default::default());
                }
                let inner = ret.get_mut(pkey).unwrap();
                for key in keys_split {
                    if let Some(Ok(value)) = values_split.next().map(|v| v.parse()) {
                        inner.insert(key.to_owned(), value);
                    }
                }
            }
            keys.clear();
            values.clear();
        }
    }
    Ok(ret)
}

pub fn default_percentiles() -> Vec<f64> {
    vec![1.0, 10.0, 50.0, 90.0, 99.0]
}
