// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum XfsStatistic {
    ReadLatency,
    WriteLatency,
    OpenLatency,
    FsyncLatency,
}

impl XfsStatistic {
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

impl Statistic for XfsStatistic {
    fn name(&self) -> &str {
        match self {
            Self::ReadLatency => "xfs/read/latency",
            Self::WriteLatency => "xfs/write/latency",
            Self::OpenLatency => "xfs/open/latency",
            Self::FsyncLatency => "xfs/fsync/latency",
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::ReadLatency => Some("latency of xfs read operations"),
            Self::WriteLatency => Some("latency of xfs write operations"),
            Self::OpenLatency => Some("latency of xfs open operations"),
            Self::FsyncLatency => Some("latency of xfs fsync operations"),
        }
    }

    fn unit(&self) -> Option<&str> {
        Some("nanoseconds")
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Distribution
    }
}
