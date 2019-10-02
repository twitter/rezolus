// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(dead_code)]

mod http;

pub use self::http::Http;

use metrics::*;

use std::fs::{File, OpenOptions};
use std::io::Write;

pub struct StatsLog {
    file: File,
    metrics: Metrics<AtomicU32>,
    count_label: Option<String>,
}

impl StatsLog {
    pub fn new(file: &str, metrics: Metrics<AtomicU32>, count_label: Option<&str>) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file)
            .expect("Failed to open file");
        Self {
            file,
            metrics,
            count_label: count_label.map(std::string::ToString::to_string),
        }
    }

    pub fn print(&mut self) {
        let current = self.metrics.readings();
        let mut data = Vec::new();
        for reading in current {
            let label = reading.label();
            let output = reading.output();
            let value = reading.value();
            match output {
                Output::Counter => {
                    if let Some(ref count_label) = self.count_label {
                        data.push(format!("{}/{}: {}", label, count_label, value));
                    } else {
                        data.push(format!("{}: {}", label, value));
                    }
                }
                Output::Percentile(percentile) => match percentile {
                    Percentile::Minimum => {
                        data.push(format!("{}/minimum/value: {}", label, value));
                    }
                    Percentile::Maximum => {
                        data.push(format!("{}/maximum/value: {}", label, value));
                    }
                    _ => {
                        data.push(format!("{}/histogram/{}: {}", label, percentile, value));
                    }
                },
                Output::MaxPointTime => {
                    // we have point's ns since X and current timespec and current ns sinc X
                    let point_ns = value;
                    let now_timespec = time::get_time();
                    let now_ns = time::precise_time_ns();

                    // find the number of NS in the past for point
                    let delta_ns = now_ns - point_ns;
                    let point_timespec =
                        now_timespec - time::Duration::nanoseconds(delta_ns as i64);

                    // convert to UTC
                    let point_utc = time::at_utc(point_timespec);
                    // calculate offset from the top of the minute
                    let offset = point_utc.tm_sec as u64 * 1_000_000_000 + point_utc.tm_nsec as u64;
                    let offset_ms = (offset as f64 / 1_000_000.0).floor() as usize;
                    data.push(format!("{}/maximum/offset_ms: {}", label, offset_ms));
                }
                _ => {
                    continue;
                }
            }
        }
        data.sort();
        let time = time::now_utc();
        let _ = self.file.write(format!("{}: ", time.rfc3339()).as_bytes());
        let _ = self.file.write(data.join(", ").as_bytes());
        let _ = self.file.write(b"\n");
    }

    pub fn run(&mut self) {
        let time = time::now_utc();
        let offset = time.tm_sec;
        let delay = 60 - offset;
        std::thread::sleep(std::time::Duration::new(delay as u64, 0));
        loop {
            std::thread::sleep(std::time::Duration::new(60, 0));
            self.print();
        }
    }
}
