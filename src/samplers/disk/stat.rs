// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use metrics::Statistic;
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{EnumString, IntoStaticStr};

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq, Hash, Serialize,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum DiskStatistic {
    #[strum(serialize = "disk/read/bytes")]
    BandwidthRead,
    #[strum(serialize = "disk/write/bytes")]
    BandwidthWrite,
    #[strum(serialize = "disk/discard/bytes")]
    BandwidthDiscard,
    #[strum(serialize = "disk/read/operations")]
    OperationsRead,
    #[strum(serialize = "disk/write/operations")]
    OperationsWrite,
    #[strum(serialize = "disk/discard/operations")]
    OperationsDiscard,
    #[strum(serialize = "disk/read/latency")]
    LatencyRead,
    #[strum(serialize = "disk/write/latency")]
    LatencyWrite,
    #[strum(serialize = "disk/read/device_latency")]
    DeviceLatencyRead,
    #[strum(serialize = "disk/write/device_latency")]
    DeviceLatencyWrite,
    #[strum(serialize = "disk/read/queue_latency")]
    QueueLatencyRead,
    #[strum(serialize = "disk/write/queue_latency")]
    QueueLatencyWrite,
    #[strum(serialize = "disk/read/io_size")]
    IoSizeRead,
    #[strum(serialize = "disk/write/io_size")]
    IoSizeWrite,
}

impl DiskStatistic {
    pub fn diskstat_field(self) -> Option<usize> {
        match self {
            Self::BandwidthRead => Some(2),
            Self::BandwidthWrite => Some(6),
            Self::BandwidthDiscard => Some(13),
            Self::OperationsRead => Some(0),
            Self::OperationsWrite => Some(4),
            Self::OperationsDiscard => Some(11),
            _ => None,
        }
    }

    pub fn bpf_table(self) -> Option<&'static str> {
        match self {
            Self::LatencyRead => Some("latency_read"),
            Self::LatencyWrite => Some("latency_write"),
            Self::DeviceLatencyRead => Some("device_latency_read"),
            Self::DeviceLatencyWrite => Some("device_latency_write"),
            Self::QueueLatencyRead => Some("queue_latency_read"),
            Self::QueueLatencyWrite => Some("queue_latency_write"),
            Self::IoSizeRead => Some("io_size_read"),
            Self::IoSizeWrite => Some("io_size_write"),
            _ => None,
        }
    }
}

impl Statistic for DiskStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> metrics::Source {
        if self.bpf_table().is_some() {
            metrics::Source::Distribution
        } else {
            metrics::Source::Counter
        }
    }
}

impl TryFrom<&str> for DiskStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        DiskStatistic::from_str(s)
    }
}
