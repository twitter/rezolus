// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;
use metrics::Statistic;
use serde_derive::*;
use strum::ParseError;
use strum_macros::*;

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq, Hash, Serialize,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum Ext4Statistic {
    #[strum(serialize = "ext4/read/latency")]
    ReadLatency,
    #[strum(serialize = "ext4/write/latency")]
    WriteLatency,
    #[strum(serialize = "ext4/open/latency")]
    OpenLatency,
    #[strum(serialize = "ext4/fsync/latency")]
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
        (*self).into()
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

impl TryFrom<&str> for Ext4Statistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ext4Statistic::from_str(s)
    }
}
