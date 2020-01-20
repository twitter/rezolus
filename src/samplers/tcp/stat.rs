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
}

impl TcpStatistic {
    pub fn keys(self) -> Option<(&'static str, &'static str)> {
        match self {
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
            Self::ReceiveSegments => "tcp segments received",
            Self::TransmitSegments => "tcp segments transmitted",
            Self::ReceivePruneCalled => "number of times pruning has been run on the receive queue",
            Self::ReceiveCollapsed => {
                "tcp packets collapsed in receive queue due to low socket buffer"
            }
            Self::Retransmits => "tcp segments retransmitted",
            Self::ReceiveChecksumErrors => "tcp segments received with checksum errors",
            Self::TransmitResets => "tcp segments transmitted with the RST flag",
            Self::ReceiveErrors => "tcp segments received in error",
            Self::SyncookiesSent => "number of sent SYN cookies",
            Self::SyncookiesRecieved => "number of received SYN cookies",
            Self::SyncookiesFailed => "number of failed SYN cookies",
            Self::ReceivePruned => "tcp packets pruned from receive queue",
            Self::ReceiveOfoPruned => {
                "tcp packets dropped from out-of-order queue due to low socket buffer"
            }
            Self::TransmitDelayedAcks => "number of delayed ACKs sent",
            Self::ReceiveListenOverflows => "number of times the listen queue of a socket overflowed",
            Self::ReceiveListenDrops => "number of SYNs to LISTEN sockets dropped",
        })
    }

    fn unit(&self) -> Option<&str> {
        match self {
            Self::ConnectLatency => Some("nanoseconds"),
            Self::ReceiveSegments | Self::TransmitSegments => Some("segments"),
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
