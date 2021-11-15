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
    pub fn bpf_probes_required(self) -> Vec<FunctionProbe> {
        // define the unique probes below.
        let file_read_probe = FunctionProbe {
            name: String::from("xfs_file_read_iter"),
            handler: String::from("trace_entry"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let file_write_probe = FunctionProbe {
            name: String::from("xfs_file_write_iter"),
            handler: String::from("trace_entry"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let file_open_probe = FunctionProbe {
            name: String::from("xfs_file_open"),
            handler: String::from("trace_entry"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let file_sync_probe = FunctionProbe {
            name: String::from("xfs_file_fsync"),
            handler: String::from("trace_entry"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let file_read_ret_probe = FunctionProbe {
            name: String::from("xfs_file_read_iter"),
            handler: String::from("trace_read_return"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let file_write_ret_probe = FunctionProbe {
            name: String::from("xfs_file_write_iter"),
            handler: String::from("trace_write_return"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let file_open_ret_probe = FunctionProbe {
            name: String::from("xfs_file_open"),
            handler: String::from("trace_open_return"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let file_sync_ret_probe = FunctionProbe {
            name: String::from("xfs_file_fsync"),
            handler: String::from("trace_fsync_return"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };

        // specify what probes are required for each telemetry.
        match self {
            Self::ReadLatency => [file_read_probe, file_read_ret_probe].to_vec(),
            Self::WriteLatency => [file_write_probe, file_write_ret_probe].to_vec(),
            Self::OpenLatency => [file_open_probe, file_open_ret_probe].to_vec(),
            Self::FsyncLatency => [file_sync_probe, file_sync_ret_probe].to_vec(),
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

impl TryFrom<&str> for XfsStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        XfsStatistic::from_str(s)
    }
}
