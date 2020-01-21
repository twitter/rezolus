// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::io::BufRead;

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
