// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use metrics::Statistic;
use serde_derive::*;
use std::str::FromStr;
use strum::ParseError;
use strum_macros::*;

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq, Hash, Serialize,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum NetworkStatistic {
    #[strum(serialize = "network/receive/bytes")]
    RxBytes,
    #[strum(serialize = "network/receive/packets")]
    RxPackets,
    #[strum(serialize = "network/receive/errors")]
    RxErrors,
    #[strum(serialize = "network/receive/drops")]
    RxDrops,
    #[strum(serialize = "network/receive/fifo")]
    RxFifo,
    #[strum(serialize = "network/receive/frame")]
    RxFrame,
    #[strum(serialize = "network/receive/compressed")]
    RxCompressed,
    #[strum(serialize = "network/receive/multicast")]
    RxMulticast,
    #[strum(serialize = "network/transmit/bytes")]
    TxBytes,
    #[strum(serialize = "network/transmit/packets")]
    TxPackets,
    #[strum(serialize = "network/transmit/errors")]
    TxErrors,
    #[strum(serialize = "network/transmit/drops")]
    TxDrops,
    #[strum(serialize = "network/transmit/fifo")]
    TxFifo,
    #[strum(serialize = "network/transmit/collisions")]
    TxCollisions,
    #[strum(serialize = "network/transmit/carrier")]
    TxCarrier,
    #[strum(serialize = "network/transmit/compressed")]
    TxCompressed,
    #[strum(serialize = "network/receive/size")]
    RxSize,
    #[strum(serialize = "network/transmit/size")]
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
        (*self).into()
    }

    fn source(&self) -> metrics::Source {
        if self.ebpf_table().is_some() {
            metrics::Source::Distribution
        } else {
            metrics::Source::Counter
        }
    }
}

impl TryFrom<&str> for NetworkStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        NetworkStatistic::from_str(s)
    }
}
