// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;

use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

pub mod bpf;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

pub const SECOND: u64 = 1_000 * MILLISECOND;
pub const MILLISECOND: u64 = 1_000 * MICROSECOND;
pub const MICROSECOND: u64 = 1_000 * NANOSECOND;
pub const NANOSECOND: u64 = 1;

pub const TERABIT: u64 = 1_000 * GIGABIT;
pub const GIGABIT: u64 = 1_000 * MEGABIT;
pub const MEGABIT: u64 = 1_000 * KILOBIT;
pub const KILOBIT: u64 = 1_000 * BIT;
pub const BIT: u64 = 1;

pub const TEBIBYTE: u64 = 1_024 * GIBIBYTE;
pub const GIBIBYTE: u64 = 1_024 * MEBIBYTE;
pub const MEBIBYTE: u64 = 1_024 * KIBIBYTE;
pub const KIBIBYTE: u64 = 1_024 * BYTE;
pub const BYTE: u64 = 1;

#[allow(dead_code)]
pub const SAMPLE_PERIOD: u64 = 1; 

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
pub async fn nested_map_from_file<T: AsRef<Path>>(
    path: T,
) -> Result<HashMap<String, HashMap<String, u64>>, std::io::Error> {
    let mut ret = HashMap::<String, HashMap<String, u64>>::new();
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    while let Some(keys) = lines.next_line().await? {
        if let Some(values) = lines.next_line().await? {
            let keys: Vec<&str> = keys.trim().split_whitespace().collect();
            let values: Vec<&str> = values.trim().split_whitespace().collect();
            if let Some(pkey) = keys.get(0).map(|v| (*v).to_string()) {
                if !ret.contains_key(&pkey) {
                    ret.insert(pkey.clone(), Default::default());
                }
                let inner = ret.get_mut(&pkey).unwrap();
                for (i, key) in keys.iter().enumerate().skip(1) {
                    let value: u64 = values.get(i).unwrap_or(&"0").parse().unwrap_or(0);
                    inner.insert((*key).to_string(), value);
                }
            }
        }
    }
    Ok(ret)
}
