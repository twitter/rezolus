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
pub enum TcpStatistic {
    #[strum(serialize = "tcp/connect/latency")]
    ConnectLatency,
    #[strum(serialize = "tcp/receive/segment")]
    ReceiveSegments,
    #[strum(serialize = "tcp/transmit/segment")]
    TransmitSegments,
    #[strum(serialize = "tcp/receive/prune_called")]
    ReceivePruneCalled,
    #[strum(serialize = "tcp/receive/collapsed")]
    ReceiveCollapsed,
    #[strum(serialize = "tcp/transmit/retransmit")]
    Retransmits,
    #[strum(serialize = "tcp/receive/checksum_error")]
    ReceiveChecksumErrors,
    #[strum(serialize = "tcp/transmit/reset")]
    TransmitResets,
    #[strum(serialize = "tcp/receive/error")]
    ReceiveErrors,
    #[strum(serialize = "tcp/syncookies/sent")]
    SyncookiesSent,
    #[strum(serialize = "tcp/syncookies/received")]
    SyncookiesRecieved,
    #[strum(serialize = "tcp/syncookies/failed")]
    SyncookiesFailed,
    #[strum(serialize = "tcp/receive/pruned")]
    ReceivePruned,
    #[strum(serialize = "tcp/receive/ofo_pruned")]
    ReceiveOfoPruned,
    #[strum(serialize = "tcp/transmit/delayed_ack")]
    TransmitDelayedAcks,
    #[strum(serialize = "tcp/receive/listen_overflows")]
    ReceiveListenOverflows,
    #[strum(serialize = "tcp/receive/listen_drops")]
    ReceiveListenDrops,
    #[strum(serialize = "tcp/abort/failed")]
    AbortFailed,
    #[strum(serialize = "tcp/abort/on_close")]
    AbortOnClose,
    #[strum(serialize = "tcp/abort/on_data")]
    AbortOnData,
    #[strum(serialize = "tcp/abort/on_linger")]
    AbortOnLinger,
    #[strum(serialize = "tcp/abort/on_memory")]
    AbortOnMemory,
    #[strum(serialize = "tcp/abort/on_timeout")]
    AbortOnTimeout,
    #[strum(serialize = "tcp/srtt")]
    SmoothedRoundTripTime,
    #[strum(serialize = "tcp/jitter")]
    Jitter,
    #[strum(serialize = "tcp/connection/accepted")]
    ConnectionAccepted,
    #[strum(serialize = "tcp/connection/initiated")]
    ConnectionInitiated,
    #[strum(serialize = "tcp/drop")]
    Drop,
}

impl TcpStatistic {
    pub fn keys(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::AbortFailed => Some(("TcpExt:", "TCPAbortFailed")),
            Self::AbortOnClose => Some(("TcpExt:", "TCPAbortOnClose")),
            Self::AbortOnData => Some(("TcpExt:", "TCPAbortOnData")),
            Self::AbortOnLinger => Some(("TcpExt:", "TCPAbortOnLinger")),
            Self::AbortOnMemory => Some(("TcpExt:", "TCPAbortOnMemory")),
            Self::AbortOnTimeout => Some(("TcpExt:", "TCPAbortOnTimeout")),
            Self::ReceiveSegments => Some(("Tcp:", "InSegs")),
            Self::TransmitSegments => Some(("Tcp:", "OutSegs")),
            Self::ReceivePruneCalled => Some(("TcpExt:", "PruneCalled")),
            Self::ReceiveCollapsed => Some(("TcpExt:", "TCPRcvCollapsed")),
            Self::Retransmits => Some(("Tcp:", "RetransSegs")),
            Self::ReceiveChecksumErrors => Some(("Tcp:", "InCsumErrors")),
            Self::TransmitResets => Some(("Tcp:", "OutRsts")),
            Self::ReceiveErrors => Some(("Tcp:", "InErrs")),
            Self::SyncookiesSent => Some(("TcpExt:", "SyncookiesSent")),
            Self::SyncookiesRecieved => Some(("TcpExt:", "SyncookiesRecv")),
            Self::SyncookiesFailed => Some(("TcpExt:", "SyncookiesFailed")),
            Self::ReceivePruned => Some(("TcpExt:", "RcvPruned")),
            Self::ReceiveOfoPruned => Some(("TcpExt:", "OfoPruned")),
            Self::TransmitDelayedAcks => Some(("TcpExt:", "DelayedACKs")),
            Self::ReceiveListenOverflows => Some(("TcpExt:", "ListenOverflows")),
            Self::ReceiveListenDrops => Some(("TcpExt:", "ListenDrops")),
            _ => None,
        }
    }

    pub fn bpf_table(self) -> Option<&'static str> {
        match self {
            Self::ConnectLatency => Some("connlat"),
            Self::SmoothedRoundTripTime => Some("srtt"),
            Self::Jitter => Some("jitter"),
            Self::ConnectionAccepted => Some("conn_accepted"),
            Self::ConnectionInitiated => Some("conn_initiated"),
            Self::Drop => Some("drop"),
            _ => None,
        }
    }

    #[cfg(feature = "bpf")]
    pub fn bpf_probes_required(self) -> Vec<FunctionProbe> {
        // define the unique probes below.
        let tcp_connect_v4_probe = FunctionProbe {
            name: String::from("tcp_v4_connect"),
            handler: String::from("trace_connect"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let tcp_connect_v6_probe = FunctionProbe {
            name: String::from("tcp_v6_connect"),
            handler: String::from("trace_connect"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let tcp_connect_v4_ret_probe = FunctionProbe {
            name: String::from("tcp_v4_connect"),
            handler: String::from("trace_connect_return"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let tcp_connect_v6_ret_probe = FunctionProbe {
            name: String::from("tcp_v6_connect"),
            handler: String::from("trace_connect_return"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let tcp_rcv_state_process_probe = FunctionProbe {
            name: String::from("tcp_rcv_state_process"),
            handler: String::from("trace_tcp_rcv_state_process"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let tcp_rcv_established_probe = FunctionProbe {
            name: String::from("tcp_rcv_established"),
            handler: String::from("trace_tcp_rcv"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let inet_csk_accept_ret_probe = FunctionProbe {
            name: String::from("inet_csk_accept"),
            handler: String::from("trace_inet_socket_accept_return"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let tcp_set_state_probe = FunctionProbe {
            name: String::from("tcp_set_state"),
            handler: String::from("trace_tcp_set_state"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let tcp_finish_connect_ret_probe = FunctionProbe {
            name: String::from("tcp_finish_connect"),
            handler: String::from("trace_finish_connect"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let tcp_drop_probe = FunctionProbe {
            name: String::from("tcp_drop"),
            handler: String::from("trace_tcp_drop"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        // specify what probes are required for each telemtry.
        match self {
            Self::ConnectLatency => [
                tcp_connect_v4_probe,
                tcp_connect_v6_probe,
                tcp_rcv_state_process_probe,
            ]
            .to_vec(),
            Self::SmoothedRoundTripTime | Self::Jitter => [tcp_rcv_established_probe].to_vec(),
            Self::ConnectionAccepted => [inet_csk_accept_ret_probe, tcp_set_state_probe].to_vec(),
            Self::ConnectionInitiated => [
                tcp_connect_v4_probe,
                tcp_connect_v6_probe,
                tcp_connect_v4_ret_probe,
                tcp_connect_v6_ret_probe,
                tcp_finish_connect_ret_probe,
                tcp_set_state_probe,
            ]
            .to_vec(),
            Self::Drop => [tcp_drop_probe].to_vec(),
            _ => Vec::new(),
        }
    }
}

impl Statistic<AtomicU64, AtomicU32> for TcpStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        match self.bpf_table() {
            Some("connlat") | Some("srtt") | Some("jitter") => Source::Distribution,
            _ => Source::Counter,
        }
    }
}

impl TryFrom<&str> for TcpStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        TcpStatistic::from_str(s)
    }
}
