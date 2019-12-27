// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Ext4Statistic {
    ReadLatency,
    WriteLatency,
    OpenLatency,
    FsyncLatency,
}

impl Ext4Statistic {
    #[allow(dead_code)]
    pub fn ebpf_table(self) -> Option<&'static str> {
        match self {
            Self::ReadLatency => Some("read"),
            Self::WriteLatency => Some("write"),
            Self::OpenLatency => Some("open"),
            Self::FsyncLatency => Some("fsync"),
        }
    }
}

impl Statistic for Ext4Statistic {
    fn name(&self) -> &str {
        match self {
            Self::ReadLatency => "ext4/read/latency",
            Self::WriteLatency => "ext4/write/latency",
            Self::OpenLatency => "ext4/open/latency",
            Self::FsyncLatency => "ext4/fsync/latency",
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::ReadLatency => Some("latency of ext4 read operations"),
            Self::WriteLatency => Some("latency of ext4 write operations"),
            Self::OpenLatency => Some("latency of ext4 open operations"),
            Self::FsyncLatency => Some("latency of ext4 fsync operations"),
        }
    }

    fn unit(&self) -> Option<&str> {
        Some("nanoseconds")
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Distribution
    }
}
