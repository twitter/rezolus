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

    #[cfg(feature = "bpf")]
    pub fn bpf_probes_required(self) -> Vec<FunctionProbe> {
        // define the unique probes below.
        let generic_file_read_probe = FunctionProbe {
            name: String::from("generic_file_read_iter"),
            handler: String::from("trace_read_entry"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let ext4_file_write_probe = FunctionProbe {
            name: String::from("ext4_file_write_iter"),
            handler: String::from("trace_entry"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let ext4_file_open_probe = FunctionProbe {
            name: String::from("ext4_file_open"),
            handler: String::from("trace_entry"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let ext4_sync_file_probe = FunctionProbe {
            name: String::from("ext4_sync_file"),
            handler: String::from("trace_entry"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let generic_file_read_ret_probe = FunctionProbe {
            name: String::from("generic_file_read_iter"),
            handler: String::from("trace_read_return"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let ext4_file_write_ret_probe = FunctionProbe {
            name: String::from("ext4_file_write_iter"),
            handler: String::from("trace_write_return"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let ext4_file_open_ret_probe = FunctionProbe {
            name: String::from("ext4_file_open"),
            handler: String::from("trace_open_return"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let ext4_sync_file_ret_probe = FunctionProbe {
            name: String::from("ext4_sync_file"),
            handler: String::from("trace_fsync_return"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };

        // specify what probes are required for each telemetry.
        match self {
            Self::ReadLatency => [generic_file_read_probe, generic_file_read_ret_probe].to_vec(),
            Self::WriteLatency => [ext4_file_write_probe, ext4_file_write_ret_probe].to_vec(),
            Self::OpenLatency => [ext4_file_open_probe, ext4_file_open_ret_probe].to_vec(),
            Self::FsyncLatency => [ext4_sync_file_probe, ext4_sync_file_ret_probe].to_vec(),
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
