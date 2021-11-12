// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use rustcommon_metrics::*;
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

#[cfg(feature = "bpf")]
use crate::common::bpf::*;

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

    #[cfg(feature = "bpf")]
    pub fn bpf_probes_required(self) -> Vec<FunctionProbe> {
        // define the unique probes below.
        let pid_start_probe = FunctionProbe {
            name: String::from("blk_account_io_start"),
            handler: String::from("trace_pid_start"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let request_start_probe = FunctionProbe {
            name: String::from("blk_start_request"),
            handler: String::from("trace_req_start"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let request_mq_start_request_probe = FunctionProbe {
            name: String::from("blk_mq_start_request"),
            handler: String::from("trace_req_start"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let pid_done_probe = FunctionProbe {
            name: String::from("blk_account_io_done"),
            handler: String::from("do_count"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let pid_completion_probe = FunctionProbe {
            name: String::from("blk_account_io_completion"),
            handler: String::from("do_count"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };

        // specify what probes are required for each telemetry.
        match self {
            Self::LatencyRead | Self::LatencyWrite | Self::IoSizeRead | Self::IoSizeWrite => {
                [pid_start_probe, pid_done_probe, pid_completion_probe].to_vec()
            }
            Self::DeviceLatencyRead | Self::DeviceLatencyWrite => [
                request_start_probe,
                request_mq_start_request_probe,
                pid_done_probe,
                pid_completion_probe,
            ]
            .to_vec(),
            Self::QueueLatencyRead | Self::QueueLatencyWrite => [
                pid_start_probe,
                request_start_probe,
                request_mq_start_request_probe,
            ]
            .to_vec(),
            _ => Vec::new(),
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
