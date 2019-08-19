// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(dead_code)]

extern crate sysconf;

pub mod file;
pub mod http;
pub mod kernel_version;
pub mod net;

pub use self::http::http_get;

use logger::*;
use metrics::Percentile;

use std::io::BufRead;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

// units
pub const THOUSAND: u64 = 1_000;
pub const MILLION: u64 = 1_000_000;
pub const BILLION: u64 = 1_000_000_000;
pub const TRILLION: u64 = 1_000_000_000_000;
pub const MICROSECOND: u64 = THOUSAND; // 1US in NS
pub const MILLISECOND: u64 = MILLION; // 1MS in NS
pub const SECOND: u64 = BILLION; // 1S in NS
pub const MINUTE: u64 = 60 * SECOND;
pub const GIGABYTE: u64 = BILLION;
pub const TERABYTE: u64 = TRILLION;

pub fn pagesize() -> u64 {
    sysconf::page::pagesize() as u64
}

pub fn nanos_per_tick() -> u64 {
    let ticks_per_second = sysconf::raw::sysconf(sysconf::raw::SysconfVariable::ScClkTck)
        .expect("Failed to get Clock Ticks per Second") as u64;
    SECOND / ticks_per_second
}

// values
pub const CONTAINER_REFRESH: u64 = MINUTE;
pub const HTTP_TIMEOUT: u64 = 500 * MILLISECOND;
pub const POLL_DELAY: u64 = 50 * MILLISECOND;
pub const SECTOR_SIZE: u64 = 512; //bytes per sector

// reported percentiles
pub const PERCENTILES: &[Percentile] = &[
    Percentile::p50,
    Percentile::p75,
    Percentile::p90,
    Percentile::p99,
    Percentile::Maximum,
];

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
