// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::metrics::*;
use serde_derive::{Deserialize, Serialize};
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
#[allow(clippy::enum_variant_names)]
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
    pub fn bpf_probes_required(self) -> Vec<Probe> {
        // define the unique probes below.
        let generic_file_read_probe = Probe {
            name: "generic_file_read_iter".to_string(),
            handler: "trace_read_entry".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let ext4_file_write_probe = Probe {
            name: "ext4_file_write_iter".to_string(),
            handler: "trace_entry".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let ext4_file_open_probe = Probe {
            name: "ext4_file_open".to_string(),
            handler: "trace_entry".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let ext4_sync_file_probe = Probe {
            name: "ext4_sync_file".to_string(),
            handler: "trace_entry".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let generic_file_read_ret_probe = Probe {
            name: "generic_file_read_iter".to_string(),
            handler: "trace_read_return".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let ext4_file_write_ret_probe = Probe {
            name: "ext4_file_write_iter".to_string(),
            handler: "trace_write_return".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let ext4_file_open_ret_probe = Probe {
            name: "ext4_file_open".to_string(),
            handler: "trace_open_return".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let ext4_sync_file_ret_probe = Probe {
            name: "ext4_sync_file".to_string(),
            handler: "trace_fsync_return".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };

        // specify what probes are required for each telemetry.
        match self {
            Self::ReadLatency => vec![generic_file_read_probe, generic_file_read_ret_probe],
            Self::WriteLatency => vec![ext4_file_write_probe, ext4_file_write_ret_probe],
            Self::OpenLatency => vec![ext4_file_open_probe, ext4_file_open_ret_probe],
            Self::FsyncLatency => vec![ext4_sync_file_probe, ext4_sync_file_ret_probe],
        }
    }
}

impl Statistic for Ext4Statistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        Source::Distribution
    }
}
