// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::net;

use logger::*;

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

    pub fn get_statistic(&self, statistic: &Statistic) -> Result<u64, ()> {
        if let Some(name) = &self.name {
            net::read_network_stat(&name, statistic.as_str())
        } else {
            Err(())
        }
    }
}

pub enum Statistic {
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

impl Statistic {
    pub fn from_str(statistic: &str) -> Result<Statistic, ()> {
        match statistic {
            "rx_bytes" => Ok(Statistic::RxBytes),
            "rx_crc_errors" => Ok(Statistic::RxCrcErrors),
            "rx_dropped" => Ok(Statistic::RxDropped),
            "rx_discards_phy" => Ok(Statistic::RxDiscardsPhy),
            "rx_errors" => Ok(Statistic::RxErrors),
            "rx_fifo_errors" => Ok(Statistic::RxFifoErrors),
            "rx_missed_errors" => Ok(Statistic::RxMissedErrors),
            "rx_packets" => Ok(Statistic::RxPackets),
            "tx_bytes" => Ok(Statistic::TxBytes),
            "tx_discards_phy" => Ok(Statistic::TxDiscardsPhy),
            "tx_dropped" => Ok(Statistic::TxDropped),
            "tx_errors" => Ok(Statistic::TxErrors),
            "tx_fifo_errors" => Ok(Statistic::TxFifoErrors),
            "tx_packets" => Ok(Statistic::TxPackets),
            _ => {
                debug!("Unknown Statistic: {}", statistic);
                Err(())
            }
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Statistic::RxBytes => "rx_bytes",
            Statistic::RxCrcErrors => "rx_crc_errors",
            Statistic::RxDropped => "rx_dropped",
            Statistic::RxDiscardsPhy => "rx_discards_phy",
            Statistic::RxErrors => "rx_errors",
            Statistic::RxFifoErrors => "rx_fifo_errors",
            Statistic::RxMissedErrors => "rx_missed_errors",
            Statistic::RxPackets => "rx_packets",
            Statistic::TxBytes => "tx_bytes",
            Statistic::TxDiscardsPhy => "tx_discards_phy",
            Statistic::TxDropped => "tx_dropped",
            Statistic::TxErrors => "tx_errors",
            Statistic::TxFifoErrors => "tx_fifo_errors",
            Statistic::TxPackets => "tx_packets",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Statistic::RxBytes => "network/receive/bytes",
            Statistic::RxCrcErrors => "network/receive/errors/crc",
            Statistic::RxDiscardsPhy => "network/receive/errors/discards_phy",
            Statistic::RxDropped => "network/receive/dropped",
            Statistic::RxErrors => "network/receive/errors/total",
            Statistic::RxFifoErrors => "network/receive/errors/fifo",
            Statistic::RxMissedErrors => "network/receive/errors/missed",
            Statistic::RxPackets => "network/receive/packets",
            Statistic::TxBytes => "network/transmit/bytes",
            Statistic::TxDiscardsPhy => "network/transmit/errors/discards_phy",
            Statistic::TxDropped => "network/transmit/dropped",
            Statistic::TxErrors => "network/transmit/errors/total",
            Statistic::TxFifoErrors => "network/transmit/errors/fifo",
            Statistic::TxPackets => "network/transmit/packets",
        }
    }
}
