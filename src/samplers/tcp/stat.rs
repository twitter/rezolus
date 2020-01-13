// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
use serde_derive::*;
use std::convert::TryFrom;
use std::str::FromStr;
use strum::ParseError;
use strum_macros::*;

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq, Hash, Serialize,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum TcpStatistic {
    #[strum(serialize = "tcp/connect/latency")]
    ConnectLatency,
    #[strum(serialize = "tcp/receive/segment")]
    RxSegments,
    #[strum(serialize = "tcp/transmit/segment")]
    TxSegments,
    #[strum(serialize = "tcp/receive/prune_called")]
    PruneCalled,
    #[strum(serialize = "tcp/receive/collapsed")]
    ReceiveCollapsed,
    #[strum(serialize = "tcp/transmit/retransmit")]
    Retransmits,
    #[strum(serialize = "tcp/receive/checksum_error")]
    RxChecksumErrors,
    #[strum(serialize = "tcp/transmit/reset")]
    TxResets,
    #[strum(serialize = "tcp/receive/error")]
    RxErrors,
    #[strum(serialize = "tcp/syncookies/sent")]
    SyncookiesSent,
    #[strum(serialize = "tcp/syncookies/received")]
    SyncookiesRecieved,
    #[strum(serialize = "tcp/syncookies/failed")]
    SyncookiesFailed,
    #[strum(serialize = "tcp/receive/pruned")]
    ReceivePruned,
    #[strum(serialize = "tcp/receive/ofo_pruned")]
    OfoPruned,
    #[strum(serialize = "tcp/transmit/delayed_ack")]
    DelayedAcks,
    #[strum(serialize = "tcp/receive/listen_overflows")]
    ListenOverflows,
    #[strum(serialize = "tcp/receive/listen_drops")]
    ListenDrops,
}

impl TcpStatistic {
    pub fn keys(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::RxSegments => Some(("Tcp:", "InSegs")),
            Self::TxSegments => Some(("Tcp:", "OutSegs")),
            Self::PruneCalled => Some(("TcpExt:", "PruneCalled")),
            Self::ReceiveCollapsed => Some(("TcpExt:", "TCPRcvCollapsed")),
            Self::Retransmits => Some(("Tcp:", "RetransSegs")),
            Self::RxChecksumErrors => Some(("Tcp:", "InCsumErrors")),
            Self::TxResets => Some(("Tcp:", "OutRsts")),
            Self::RxErrors => Some(("Tcp:", "InErrs")),
            Self::SyncookiesSent => Some(("TcpExt:", "SyncookiesSent")),
            Self::SyncookiesRecieved => Some(("TcpExt:", "SyncookiesRecv")),
            Self::SyncookiesFailed => Some(("TcpExt:", "SyncookiesFailed")),
            Self::ReceivePruned => Some(("TcpExt:", "RcvPruned")),
            Self::OfoPruned => Some(("TcpExt:", "OfoPruned")),
            Self::DelayedAcks => Some(("TcpExt:", "DelayedACKs")),
            Self::ListenOverflows => Some(("TcpExt:", "ListenOverflows")),
            Self::ListenDrops => Some(("TcpExt:", "ListenDrops")),
            _ => None,
        }
    }

    pub fn ebpf_table(self) -> Option<&'static str> {
        match self {
            Self::ConnectLatency => Some("connlat"),
            _ => None,
        }
    }
}

impl Statistic for TcpStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn description(&self) -> Option<&str> {
        Some(match self {
            Self::ConnectLatency => "latency of active tcp connect",
            Self::RxSegments => "tcp segments received",
            Self::TxSegments => "tcp segments transmitted",
            Self::PruneCalled => "number of times pruning has been run on the receive queue",
            Self::ReceiveCollapsed => {
                "tcp packets collapsed in receive queue due to low socket buffer"
            }
            Self::Retransmits => "tcp segments retransmitted",
            Self::RxChecksumErrors => "tcp segments received with checksum errors",
            Self::TxResets => "tcp segments transmitted with the RST flag",
            Self::RxErrors => "tcp segments received in error",
            Self::SyncookiesSent => "number of sent SYN cookies",
            Self::SyncookiesRecieved => "number of received SYN cookies",
            Self::SyncookiesFailed => "number of failed SYN cookies",
            Self::ReceivePruned => "tcp packets pruned from receive queue",
            Self::OfoPruned => {
                "tcp packets dropped from out-of-order queue due to low socket buffer"
            }
            Self::DelayedAcks => "number of delayed ACKs sent",
            Self::ListenOverflows => "number of times the listen queue of a socket overflowed",
            Self::ListenDrops => "number of SYNs to LISTEN sockets dropped",
        })
    }

    fn unit(&self) -> Option<&str> {
        match self {
            Self::ConnectLatency => Some("nanoseconds"),
            Self::RxSegments | Self::TxSegments => Some("segments"),
            _ => None,
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

impl TryFrom<&str> for TcpStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        TcpStatistic::from_str(s)
    }
}
