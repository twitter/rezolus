// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::net;
use crate::samplers::Statistic;

use serde_derive::*;

use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Interface {
    name: Option<String>,
    bandwidth_bytes: Option<u64>,
}

impl fmt::Display for Interface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.name {
            None => write!(f, "total"),
            Some(ref name) => write!(f, "if.{}", name),
        }
    }
}

impl Interface {
    pub fn new(name: Option<String>, bandwidth_bytes: Option<u64>) -> Self {
        Interface {
            name,
            bandwidth_bytes,
        }
    }

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn bandwidth_bytes(&self) -> Option<u64> {
        self.bandwidth_bytes
    }

    pub fn get_statistic(&self, statistic: &InterfaceStatistic) -> Result<u64, ()> {
        if let Some(name) = &self.name {
            net::read_network_stat(&name, statistic.name())
        } else {
            Err(())
        }
    }
}

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

impl fmt::Display for InterfaceStatistic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InterfaceStatistic::RxBytes => write!(f, "network/receive/bytes"),
            InterfaceStatistic::RxCrcErrors => write!(f, "network/receive/errors/crc"),
            InterfaceStatistic::RxDiscardsPhy => write!(f, "network/receive/errors/discards_phy"),
            InterfaceStatistic::RxDropped => write!(f, "network/receive/dropped"),
            InterfaceStatistic::RxErrors => write!(f, "network/receive/errors/total"),
            InterfaceStatistic::RxFifoErrors => write!(f, "network/receive/errors/fifo"),
            InterfaceStatistic::RxMissedErrors => write!(f, "network/receive/errors/missed"),
            InterfaceStatistic::RxPackets => write!(f, "network/receive/packets"),
            InterfaceStatistic::TxBytes => write!(f, "network/transmit/bytes"),
            InterfaceStatistic::TxDiscardsPhy => write!(f, "network/transmit/errors/discards_phy"),
            InterfaceStatistic::TxDropped => write!(f, "network/transmit/dropped"),
            InterfaceStatistic::TxErrors => write!(f, "network/transmit/errors/total"),
            InterfaceStatistic::TxFifoErrors => write!(f, "network/transmit/errors/fifo"),
            InterfaceStatistic::TxPackets => write!(f, "network/transmit/packets"),
        }
    }
}

impl Statistic for InterfaceStatistic {}

impl InterfaceStatistic {
    pub fn name(&self) -> &str {
        match self {
            InterfaceStatistic::RxBytes => "rx_bytes",
            InterfaceStatistic::RxCrcErrors => "rx_crc_errors",
            InterfaceStatistic::RxDropped => "rx_dropped",
            InterfaceStatistic::RxDiscardsPhy => "rx_discards_phy",
            InterfaceStatistic::RxErrors => "rx_errors",
            InterfaceStatistic::RxFifoErrors => "rx_fifo_errors",
            InterfaceStatistic::RxMissedErrors => "rx_missed_errors",
            InterfaceStatistic::RxPackets => "rx_packets",
            InterfaceStatistic::TxBytes => "tx_bytes",
            InterfaceStatistic::TxDiscardsPhy => "tx_discards_phy",
            InterfaceStatistic::TxDropped => "tx_dropped",
            InterfaceStatistic::TxErrors => "tx_errors",
            InterfaceStatistic::TxFifoErrors => "tx_fifo_errors",
            InterfaceStatistic::TxPackets => "tx_packets",
        }
    }
}
