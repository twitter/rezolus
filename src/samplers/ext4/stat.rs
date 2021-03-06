// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use rustcommon_metrics::*;
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumIter,
    EnumString,
    Eq,
    IntoStaticStr,
    PartialEq,
    Hash,
    Serialize,
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
    pub fn bpf_table(self) -> Option<&'static str> {
        match self {
            Self::ReadLatency => Some("read"),
            Self::WriteLatency => Some("write"),
            Self::OpenLatency => Some("open"),
            Self::FsyncLatency => Some("fsync"),
        }
    }
}

impl Statistic<AtomicU64, AtomicU32> for Ext4Statistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        Source::Distribution
    }
}

impl TryFrom<&str> for Ext4Statistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ext4Statistic::from_str(s)
    }
}
