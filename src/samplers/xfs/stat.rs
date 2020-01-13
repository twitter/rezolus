// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
use serde_derive::*;
use std::convert::TryFrom;
use std::str::FromStr;
use strum::ParseError;
use strum_macros::*;

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq, Hash, Serialize,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum XfsStatistic {
    #[strum(serialize = "xfs/read/latency")]
    ReadLatency,
    #[strum(serialize = "xfs/write/latency")]
    WriteLatency,
    #[strum(serialize = "xfs/open/latency")]
    OpenLatency,
    #[strum(serialize = "xfs/fsync/latency")]
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
        (*self).into()
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

impl TryFrom<&str> for XfsStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        XfsStatistic::from_str(s)
    }
}
