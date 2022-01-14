// Copyright 2019-2020 Twitter, Inc.
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
pub enum NetworkStatistic {
    #[strum(serialize = "network/receive/bytes")]
    ReceiveBytes,
    #[strum(serialize = "network/receive/packets")]
    ReceivePackets,
    #[strum(serialize = "network/receive/errors")]
    ReceiveErrors,
    #[strum(serialize = "network/receive/drops")]
    ReceiveDrops,
    #[strum(serialize = "network/receive/fifo")]
    ReceiveFifo,
    #[strum(serialize = "network/receive/frame")]
    ReceiveFrame,
    #[strum(serialize = "network/receive/compressed")]
    ReceiveCompressed,
    #[strum(serialize = "network/receive/multicast")]
    ReceiveMulticast,
    #[strum(serialize = "network/transmit/bytes")]
    TransmitBytes,
    #[strum(serialize = "network/transmit/packets")]
    TransmitPackets,
    #[strum(serialize = "network/transmit/errors")]
    TransmitErrors,
    #[strum(serialize = "network/transmit/drops")]
    TransmitDrops,
    #[strum(serialize = "network/transmit/fifo")]
    TransmitFifo,
    #[strum(serialize = "network/transmit/collisions")]
    TransmitCollisions,
    #[strum(serialize = "network/transmit/carrier")]
    TransmitCarrier,
    #[strum(serialize = "network/transmit/compressed")]
    TransmitCompressed,
    #[strum(serialize = "network/receive/size")]
    ReceiveSize,
    #[strum(serialize = "network/transmit/size")]
    TransmitSize,
}

impl NetworkStatistic {
    pub fn field_number(self) -> Option<usize> {
        match self {
            Self::ReceiveBytes => Some(1),
            Self::ReceivePackets => Some(2),
            Self::ReceiveErrors => Some(3),
            Self::ReceiveDrops => Some(4),
            Self::ReceiveFifo => Some(5),
            Self::ReceiveFrame => Some(6),
            Self::ReceiveCompressed => Some(7),
            Self::ReceiveMulticast => Some(8),
            Self::TransmitBytes => Some(9),
            Self::TransmitPackets => Some(10),
            Self::TransmitErrors => Some(11),
            Self::TransmitDrops => Some(12),
            Self::TransmitFifo => Some(13),
            Self::TransmitCollisions => Some(14),
            Self::TransmitCarrier => Some(15),
            Self::TransmitCompressed => Some(16),
            _ => None,
        }
    }

    pub fn bpf_table(self) -> Option<&'static str> {
        match self {
            Self::ReceiveSize => Some("rx_size"),
            Self::TransmitSize => Some("tx_size"),
            _ => None,
        }
    }

    #[cfg(feature = "bpf")]
    pub fn bpf_probes_required(self) -> Vec<Probe> {
        // define the unique probes below.
        let tx_tracepoint = Probe {
            name: "net_dev_queue".to_string(),
            handler: "trace_transmit".to_string(),
            probe_type: ProbeType::Tracepoint,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: Some("net".to_string()),
        };
        let rx_tracepoint = Probe {
            name: "netif_rx".to_string(),
            handler: "trace_receive".to_string(),
            probe_type: ProbeType::Tracepoint,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: Some("net".to_string()),
        };

        match self {
            Self::ReceiveSize => vec![rx_tracepoint],
            Self::TransmitSize => vec![tx_tracepoint],
            _ => Vec::new(),
        }
    }
}

impl Statistic<AtomicU64, AtomicU32> for NetworkStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        if self.bpf_table().is_some() {
            Source::Distribution
        } else {
            Source::Counter
        }
    }
}
