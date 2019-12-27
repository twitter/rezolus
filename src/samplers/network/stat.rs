// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum NetworkStatistic {
    RxBytes,
    RxPackets,
    RxErrors,
    RxDrops,
    RxFifo,
    RxFrame,
    RxCompressed,
    RxMulticast,
    TxBytes,
    TxPackets,
    TxErrors,
    TxDrops,
    TxFifo,
    TxCollisions,
    TxCarrier,
    TxCompressed,
    RxSize,
    TxSize,
}

impl NetworkStatistic {
    pub fn field_number(self) -> Option<usize> {
        match self {
            Self::RxBytes => Some(1),
            Self::RxPackets => Some(2),
            Self::RxErrors => Some(3),
            Self::RxDrops => Some(4),
            Self::RxFifo => Some(5),
            Self::RxFrame => Some(6),
            Self::RxCompressed => Some(7),
            Self::RxMulticast => Some(8),
            Self::TxBytes => Some(9),
            Self::TxPackets => Some(10),
            Self::TxErrors => Some(11),
            Self::TxDrops => Some(12),
            Self::TxFifo => Some(13),
            Self::TxCollisions => Some(14),
            Self::TxCarrier => Some(15),
            Self::TxCompressed => Some(16),
            _ => None,
        }
    }

    pub fn ebpf_table(self) -> Option<&'static str> {
        match self {
            Self::RxSize => Some("rx_size"),
            Self::TxSize => Some("tx_size"),
            _ => None,
        }
    }
}

impl Statistic for NetworkStatistic {
    fn name(&self) -> &str {
        match self {
            Self::RxBytes => "network/receive/bytes",
            Self::RxPackets => "network/receive/packets",
            Self::RxErrors => "network/receive/errors",
            Self::RxDrops => "network/receive/drops",
            Self::RxFifo => "network/receive/fifo",
            Self::RxFrame => "network/receive/frame",
            Self::RxCompressed => "network/receive/compressed",
            Self::RxMulticast => "network/receive/multicast",
            Self::TxBytes => "network/transmit/bytes",
            Self::TxPackets => "network/transmit/packets",
            Self::TxErrors => "network/transmit/errors",
            Self::TxDrops => "network/transmit/drops",
            Self::TxFifo => "network/transmit/fifo",
            Self::TxCollisions => "network/transmit/collisions",
            Self::TxCarrier => "network/transmit/carrier",
            Self::TxCompressed => "network/transmit/compressed",
            Self::RxSize => "network/receive/size",
            Self::TxSize => "network/transmit/size",
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
