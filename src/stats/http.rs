// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::MILLISECOND;

use logger::*;
use metrics::{Output, Percentile, Reading, Recorder};
use tiny_http::{Method, Response, Server};

use std::net::SocketAddr;

pub struct Http {
    recorder: Recorder<u32>,
    server: Server,
    snapshot: Vec<Reading>,
    refreshed: u64,
    count_label: Option<String>,
}

impl Http {
    pub fn new(address: SocketAddr, recorder: &Recorder<u32>, count_label: Option<&str>) -> Self {
        let server = tiny_http::Server::http(address);
        if server.is_err() {
            fatal!("Failed to open {} for HTTP Stats listener", address);
        }
        Self {
            recorder: recorder.clone(),
            server: server.unwrap(),
            snapshot: Vec::new(),
            refreshed: 0,
            count_label: count_label.map(std::string::ToString::to_string),
        }
    }

    pub fn run(&mut self) {
        let now = time::precise_time_ns();
        if now - self.refreshed > 500 * MILLISECOND {
            self.snapshot = self.recorder.readings();
            self.refreshed = time::precise_time_ns();
        }
        if let Ok(Some(request)) = self.server.try_recv() {
            let url = request.url();
            let parts: Vec<&str> = url.split('?').collect();
            let url = parts[0];
            match request.method() {
                Method::Get => match url {
                    "/" => {
                        debug!("Serving GET on index");
                        let _ = request.respond(Response::from_string(format!(
                            "Welcome to {}\nVersion: {}\n",
                            crate::config::NAME,
                            crate::config::VERSION,
                        )));
                    }
                    "/metrics" => {
                        debug!("Serving Prometheus compatible stats");
                        let _ = request.respond(Response::from_string(self.prometheus()));
                    }
                    "/metrics.json" | "/vars.json" | "/admin/metrics.json" => {
                        debug!("Serving machine readable stats");
                        let _ = request.respond(Response::from_string(self.json(false)));
                    }
                    "/vars" => {
                        debug!("Serving human readable stats");
                        let _ = request.respond(Response::from_string(self.human()));
                    }
                    url => {
                        debug!("GET on non-existent url: {}", url);
                        debug!("Serving machine readable stats");
                        let _ = request.respond(Response::from_string(self.json(false)));
                    }
                },
                method => {
                    debug!("unsupported request method: {}", method);
                    let _ = request.respond(Response::empty(404));
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    pub fn prometheus(&self) -> String {
        let mut data = Vec::new();
        for reading in &self.snapshot {
            let label = reading.label();
            let output = reading.output();
            let value = reading.value();
            match output {
                Output::Counter => {
                    if let Some(ref count_label) = self.count_label {
                        data.push(format!("{}/{} {}", label, count_label, value));
                    } else {
                        data.push(format!("{} {}", label, value));
                    }
                }
                Output::Percentile(percentile) => match percentile {
                    Percentile::Minimum => {
                        data.push(format!("{}/minimum/value {}", label, value));
                    }
                    Percentile::Maximum => {
                        data.push(format!("{}/maximum/value {}", label, value));
                    }
                    _ => {
                        data.push(format!("{}/histogram/{} {}", label, percentile, value));
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
                    let offset_ms = (offset as f64 / 1_000_000.0).floor() as u64;
                    data.push(format!("{}/maximum/offset_ms {}", label, offset_ms));
                }
                _ => {
                    continue;
                }
            }
        }
        data.sort();
        let mut content = data.join("\n");
        content += "\n";
        let parts: Vec<&str> = content.split('/').collect();
        let content = parts.join("_");
        content
    }

    pub fn human(&self) -> String {
        let mut data = Vec::new();
        for reading in &self.snapshot {
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
                    let offset_ms = (offset as f64 / 1_000_000.0).floor() as u64;
                    data.push(format!("{}/maximum/offset_ms: {}", label, offset_ms));
                }
                _ => {
                    continue;
                }
            }
        }
        data.sort();
        let mut content = data.join("\n");
        content += "\n";
        content
    }

    fn json(&self, pretty: bool) -> String {
        let mut head = "{".to_owned();
        if pretty {
            head += "\n  ";
        }
        let mut data = Vec::new();
        for reading in &self.snapshot {
            let label = reading.label();
            let output = reading.output();
            let value = reading.value();
            match output {
                Output::Counter => {
                    if let Some(ref count_label) = self.count_label {
                        data.push(format!("\"{}/{}\": {}", label, count_label, value));
                    } else {
                        data.push(format!("{}: {}", label, value));
                    }
                }
                Output::Percentile(percentile) => match percentile {
                    Percentile::Minimum => {
                        data.push(format!("\"{}/minimum/value\": {}", label, value));
                    }
                    Percentile::Maximum => {
                        data.push(format!("\"{}/maximum/value\": {}", label, value));
                    }
                    _ => {
                        data.push(format!("\"{}/histogram/{}\": {}", label, percentile, value));
                    }
                },
                Output::MaxPointTime => {
                    // we have point's ns since X and current timespec and current ns since X
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
                    let offset_ms = (offset as f64 / 1_000_000.0).floor() as u64;
                    data.push(format!("\"{}/maximum/offset_ms\": {}", label, offset_ms));
                }
                _ => {
                    continue;
                }
            }
        }
        data.sort();
        let body = if pretty {
            data.join(",\n  ")
        } else {
            data.join(",")
        };
        let mut content = head;
        content += &body;
        if pretty {
            content += "\n";
        }
        content += "}";
        content
    }
}
