// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum DiskStatistic {
    BandwidthRead,
    BandwidthWrite,
    BandwidthDiscard,
    OperationsRead,
    OperationsWrite,
    OperationsDiscard,
    LatencyRead,
    LatencyWrite,
    DeviceLatencyRead,
    DeviceLatencyWrite,
    QueueLatencyRead,
    QueueLatencyWrite,
    IoSizeRead,
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

    pub fn ebpf_table(self) -> Option<&'static str> {
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
        match self {
            Self::BandwidthDiscard => "disk/bandwidth/discard",
            Self::BandwidthRead => "disk/bandwidth/read",
            Self::BandwidthWrite => "disk/bandwidth/write",
            Self::OperationsDiscard => "disk/operations/discard",
            Self::OperationsRead => "disk/operations/read",
            Self::OperationsWrite => "disk/operations/write",
            Self::LatencyRead => "disk/latency/read",
            Self::LatencyWrite => "disk/latency/write",
            Self::DeviceLatencyRead => "disk/device_latency/read",
            Self::DeviceLatencyWrite => "disk/device_latency/write",
            Self::QueueLatencyRead => "disk/queue_latency/read",
            Self::QueueLatencyWrite => "disk/queue_latency/write",
            Self::IoSizeRead => "disk/io_size/read",
            Self::IoSizeWrite => "disk/io_size/write",
        }
    }

    fn source(&self) -> metrics::Source {
        if self.ebpf_table().is_some() {
            metrics::Source::Distribution
        } else {
            metrics::Source::Counter
        }
    }
}
