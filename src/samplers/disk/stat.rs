// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use rustcommon_metrics::*;
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

use crate::metrics::{StreamSummarizedCounter, SummarizedDistribution};
use std::collections::HashSet;
use std::time::Duration;

stats_struct! {
    pub struct DiskStats {
        pub bandwidth_read: StreamSummarizedCounter = "disk/read/bytes",
        pub bandwidth_write: StreamSummarizedCounter = "disk/write/bytes",
        pub bandwidth_discard: StreamSummarizedCounter = "disk/discard/bytes",
        pub operations_read: StreamSummarizedCounter = "disk/read/operations",
        pub operations_write: StreamSummarizedCounter = "disk/write/operations",
        pub operations_discard: StreamSummarizedCounter = "disk/discard/operations",
        pub latency_read: SummarizedDistribution = "disk/read/latency",
        pub latency_write: SummarizedDistribution = "disk/write/latency",
        pub device_latency_read: SummarizedDistribution = "disk/read/device_latency",
        pub device_latency_write: SummarizedDistribution = "disk/write/device_latency",
        pub queue_latency_read: SummarizedDistribution = "disk/read/queue_latency",
        pub queue_latency_write: SummarizedDistribution = "disk/write/queue_latency",
        pub io_size_read: SummarizedDistribution = "disk/read/io_size",
        pub io_size_write: SummarizedDistribution = "disk/write/io_size",
    }
}

impl DiskStats {
    pub fn new(capacity: usize, span: Duration, percentiles: &[f64]) -> Self {
        Self {
            bandwidth_read: StreamSummarizedCounter::new(capacity, percentiles),
            bandwidth_write: StreamSummarizedCounter::new(capacity, percentiles),
            bandwidth_discard: StreamSummarizedCounter::new(capacity, percentiles),
            operations_read: StreamSummarizedCounter::new(capacity, percentiles),
            operations_write: StreamSummarizedCounter::new(capacity, percentiles),
            operations_discard: StreamSummarizedCounter::new(capacity, percentiles),
            latency_read: SummarizedDistribution::new(span, percentiles),
            latency_write: SummarizedDistribution::new(span, percentiles),
            device_latency_read: SummarizedDistribution::new(span, percentiles),
            device_latency_write: SummarizedDistribution::new(span, percentiles),
            queue_latency_read: SummarizedDistribution::new(span, percentiles),
            queue_latency_write: SummarizedDistribution::new(span, percentiles),
            io_size_read: SummarizedDistribution::new(span, percentiles),
            io_size_write: SummarizedDistribution::new(span, percentiles),
        }
    }
}

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

impl Statistic<AtomicU64, AtomicU32> for DiskStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        if self.bpf_table().is_some() {
            Source::Distribution
        } else {
            Source::Counter
        }
    }
}

impl TryFrom<&str> for DiskStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        DiskStatistic::from_str(s)
    }
}
