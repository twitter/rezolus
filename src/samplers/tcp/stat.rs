// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum TcpStatistic {
    ConnectLatency,
    RxSegments,
    TxSegments,
    PruneCalled,
    ReceiveCollapsed,
    Retransmits,
    RxChecksumErrors,
    TxResets,
    RxErrors,
    SyncookiesSent,
    SyncookiesRecieved,
    SyncookiesFailed,
    ReceivePruned,
    OfoPruned,
    DelayedAcks,
    ListenOverflows,
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
        match self {
            Self::ConnectLatency => "tcp/connect/latency",
            Self::RxSegments => "tcp/receive/segments",
            Self::TxSegments => "tcp/transmit/segments",
            Self::PruneCalled => "tcp/receive/prune_called",
            Self::ReceiveCollapsed => "tcp/receive/collapsed",
            Self::Retransmits => "tcp/transmit/retransmits",
            Self::RxChecksumErrors => "tcp/receive/checksum_errors",
            Self::TxResets => "tcp/transmit/resets",
            Self::RxErrors => "tcp/receive/errors",
            Self::SyncookiesSent => "tcp/syncookies/sent",
            Self::SyncookiesRecieved => "tcp/syncookies/received",
            Self::SyncookiesFailed => "tcp/syncookies/failed",
            Self::ReceivePruned => "tcp/receive/pruned",
            Self::OfoPruned => "tcp/receive/ofo_pruned",
            Self::DelayedAcks => "tcp/transmit/delayed_acks",
            Self::ListenOverflows => "tcp/receive/listen_overflows",
            Self::ListenDrops => "tcp/receive/listen_drops",
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::ConnectLatency => Some("latency of active tcp connect"),
            Self::RxSegments => Some("tcp segments received"),
            Self::TxSegments => Some("tcp segments transmitted"),
            Self::PruneCalled => {
                Some("tcp packets dropped from receive queue due to socket buffer overrun")
            }
            Self::ReceiveCollapsed => {
                Some("tcp packets collapsed in receive queue due to low socket buffer")
            }
            Self::Retransmits => Some("tcp segments retransmitted"),
            Self::RxChecksumErrors => Some("tcp segments received with checksum errors"),
            Self::TxResets => Some("tcp segments transmitted with the RST flag"),
            Self::RxErrors => Some("tcp segments received in error"),
            Self::SyncookiesSent => Some("number of sent SYN cookies"),
            Self::SyncookiesRecieved => Some("number of received SYN cookies"),
            Self::SyncookiesFailed => Some("number of failed SYN cookies"),
            Self::ReceivePruned => Some("tcp packets pruned from receive queue"),
            Self::OfoPruned => {
                Some("tcp packets dropped from out-of-order queue due to low socket buffer")
            }
            Self::DelayedAcks => Some("number of delayed ACKs sent"),
            Self::ListenOverflows => {
                Some("number of times the listen queue of a socket overflowed")
            }
            Self::ListenDrops => Some("number of SYNs to LISTEN sockets dropped"),
        }
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
