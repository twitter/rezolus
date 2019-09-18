// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub enum Direction {
    Read,
    Write,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
        }
    }
}

pub enum Statistic {
    DeviceLatency(Direction),
    Latency(Direction),
    QueueLatency(Direction),
    Size(Direction),
}

impl std::fmt::Display for Statistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DeviceLatency(direction) => write!(f, "block/device_latency/{}", direction),
            Self::Latency(direction) => write!(f, "block/latency/{}", direction),
            Self::QueueLatency(direction) => write!(f, "block/queue_latency/{}", direction),
            Self::Size(direction) => write!(f, "block/size/{}", direction),
        }
    }
}

impl Statistic {
    pub fn table_name(&self) -> String {
        match self {
            Self::DeviceLatency(direction) => format!("{}_request_latency", direction),
            Self::Latency(direction) => format!("{}_rlatency", direction),
            Self::QueueLatency(direction) => format!("{}_queue_latency", direction),
            Self::Size(direction) => format!("{}_size", direction),
        }
    }
}
