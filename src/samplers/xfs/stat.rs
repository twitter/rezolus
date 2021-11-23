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
    pub fn bpf_table(self) -> Option<&'static str> {
        match self {
            Self::ReadLatency => Some("read"),
            Self::WriteLatency => Some("write"),
            Self::OpenLatency => Some("open"),
            Self::FsyncLatency => Some("fsync"),
        }
    }

    #[cfg(feature = "bpf")]
    pub fn bpf_probes_required(self) -> Vec<Probe> {
        // define the unique probes below.
        let file_read_probe = Probe {
            name: "xfs_file_read_iter".to_string(),
            handler: "trace_entry".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let file_write_probe = Probe {
            name: "xfs_file_write_iter".to_string(),
            handler: "trace_entry".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let file_open_probe = Probe {
            name: "xfs_file_open".to_string(),
            handler: "trace_entry".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let file_sync_probe = Probe {
            name: "xfs_file_fsync".to_string(),
            handler: "trace_entry".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let file_read_ret_probe = Probe {
            name: "xfs_file_read_iter".to_string(),
            handler: "trace_read_return".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let file_write_ret_probe = Probe {
            name: "xfs_file_write_iter".to_string(),
            handler: "trace_write_return".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let file_open_ret_probe = Probe {
            name: "xfs_file_open".to_string(),
            handler: "trace_open_return".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let file_sync_ret_probe = Probe {
            name: "xfs_file_fsync".to_string(),
            handler: "trace_fsync_return".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };

        // specify what probes are required for each telemetry.
        match self {
            Self::ReadLatency => vec![file_read_probe, file_read_ret_probe],
            Self::WriteLatency => vec![file_write_probe, file_write_ret_probe],
            Self::OpenLatency => vec![file_open_probe, file_open_ret_probe],
            Self::FsyncLatency => vec![file_sync_probe, file_sync_ret_probe],
        }
    }
}

impl Statistic<AtomicU64, AtomicU32> for XfsStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        Source::Distribution
    }
}
