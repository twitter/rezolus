// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum InterfaceStatistic {
    /// Indicates the number of bytes received by this network device.
    /// See the network driver for the exact meaning of when this
    /// value is incremented.
    RxBytes,
    /// Indicates the number of packets received with a CRC (FCS) error
    /// by this network device. Note that the specific meaning might
    /// depend on the MAC layer used by the interface.
    RxCrcErrors,
    /// The number of received packets dropped due to lack of buffers on a
    /// physical port. If this counter is increasing, it implies that the
    /// adapter is congested and cannot absorb the traffic coming from the
    /// network.
    RxDiscardsPhy,
    /// Indicates the number of packets received by the network device
    /// but dropped, that are not forwarded to the upper layers for
    /// packet processing. See the network driver for the exact
    /// meaning of this value.
    RxDropped,
    RxErrors,
    /// Indicates the number of receive FIFO errors seen by this
    /// network device. See the network driver for the exact
    /// meaning of this value.
    /// drivers: mlx4
    RxFifoErrors,
    /// Indicates the number of received packets that have been missed
    /// due to lack of capacity in the receive side. See the network
    /// driver for the exact meaning of this value.
    /// drivers: ixgbe
    RxMissedErrors,
    /// Indicates the total number of good packets received by this
    /// network device.
    RxPackets,
    /// Indicates the number of bytes transmitted by a network
    /// device. See the network driver for the exact meaning of this
    /// value, in particular whether this accounts for all successfully
    /// transmitted packets or all packets that have been queued for
    /// transmission.
    TxBytes,
    /// The number of transmit packets dropped due to lack of buffers on a
    /// physical port. If this counter is increasing, it implies that the
    /// adapter is congested and cannot absorb the traffic coming from the
    /// network.
    /// drivers: mlx5
    TxDiscardsPhy,
    /// Indicates the number of packets dropped during transmission.
    /// See the driver for the exact reasons as to why the packets were
    /// dropped.
    TxDropped,
    /// Indicates the number of packets in error during transmission by
    /// a network device. See the driver for the exact reasons as to
    /// why the packets were dropped.
    TxErrors,
    /// Indicates the number of packets having caused a transmit
    /// FIFO error. See the driver for the exact reasons as to why the
    /// packets were dropped.
    /// drivers: mlx4
    TxFifoErrors,
    /// Indicates the number of packets transmitted by a network
    /// device. See the driver for whether this reports the number of all
    /// attempted or successful transmissions.
    TxPackets,
}

impl std::fmt::Display for InterfaceStatistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::RxBytes => write!(f, "network/receive/bytes"),
            Self::RxCrcErrors => write!(f, "network/receive/errors/crc"),
            Self::RxDiscardsPhy => write!(f, "network/receive/errors/discards_phy"),
            Self::RxDropped => write!(f, "network/receive/dropped"),
            Self::RxErrors => write!(f, "network/receive/errors/total"),
            Self::RxFifoErrors => write!(f, "network/receive/errors/fifo"),
            Self::RxMissedErrors => write!(f, "network/receive/errors/missed"),
            Self::RxPackets => write!(f, "network/receive/packets"),
            Self::TxBytes => write!(f, "network/transmit/bytes"),
            Self::TxDiscardsPhy => write!(f, "network/transmit/errors/discards_phy"),
            Self::TxDropped => write!(f, "network/transmit/dropped"),
            Self::TxErrors => write!(f, "network/transmit/errors/total"),
            Self::TxFifoErrors => write!(f, "network/transmit/errors/fifo"),
            Self::TxPackets => write!(f, "network/transmit/packets"),
        }
    }
}

impl InterfaceStatistic {
    pub fn name(&self) -> &str {
        match self {
            Self::RxBytes => "rx_bytes",
            Self::RxCrcErrors => "rx_crc_errors",
            Self::RxDropped => "rx_dropped",
            Self::RxDiscardsPhy => "rx_discards_phy",
            Self::RxErrors => "rx_errors",
            Self::RxFifoErrors => "rx_fifo_errors",
            Self::RxMissedErrors => "rx_missed_errors",
            Self::RxPackets => "rx_packets",
            Self::TxBytes => "tx_bytes",
            Self::TxDiscardsPhy => "tx_discards_phy",
            Self::TxDropped => "tx_dropped",
            Self::TxErrors => "tx_errors",
            Self::TxFifoErrors => "tx_fifo_errors",
            Self::TxPackets => "tx_packets",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ProtocolStatistic {
    TcpInSegs,
    TcpOutSegs,
    TcpPruneCalled,
    TcpRcvCollapsed,
    TcpRetransSegs,
    UdpInDatagrams,
    UdpInErrors,
    UdpOutDatagrams,
}

impl std::fmt::Display for ProtocolStatistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::TcpInSegs => write!(f, "network/tcp/receive/segments"),
            Self::TcpOutSegs => write!(f, "network/tcp/transmit/segments"),
            Self::TcpPruneCalled => write!(f, "network/tcp/receive/prune_called"),
            Self::TcpRcvCollapsed => write!(f, "network/tcp/receive/collapsed"),
            Self::TcpRetransSegs => write!(f, "network/tcp/transmit/retranmits"),
            Self::UdpInDatagrams => write!(f, "network/udp/receive/datagrams"),
            Self::UdpInErrors => write!(f, "network/udp/receive/errors"),
            Self::UdpOutDatagrams => write!(f, "network/udp/transmit/datagrams"),
        }
    }
}

impl ProtocolStatistic {
    pub fn name(&self) -> &str {
        match self {
            Self::TcpInSegs => "InSegs",
            Self::TcpOutSegs => "OutSegs",
            Self::TcpPruneCalled => "PruneCalled",
            Self::TcpRcvCollapsed => "TCPRcvCollapsed",
            Self::TcpRetransSegs => "RetransSegs",
            Self::UdpInDatagrams => "InDatagrams",
            Self::UdpInErrors => "InErrors",
            Self::UdpOutDatagrams => "OutDatagrams",
        }
    }

    pub fn protocol(&self) -> &str {
        match self {
            Self::TcpInSegs | Self::TcpOutSegs | Self::TcpRetransSegs => "Tcp:",
            Self::TcpPruneCalled | Self::TcpRcvCollapsed => "TcpExt:",
            Self::UdpInDatagrams | Self::UdpInErrors | Self::UdpOutDatagrams => "Udp:",
        }
    }
}
