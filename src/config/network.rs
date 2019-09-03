// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;
use crate::samplers::network::interface::InterfaceStatistic;
use crate::samplers::network::protocol::ProtocolStatistic;

use atomics::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    #[serde(default = "default_enabled")]
    enabled: AtomicBool,
    #[serde(default = "default_interface_statistics")]
    interface_statistics: Vec<InterfaceStatistic>,
    #[serde(default = "default_protocol_statistics")]
    protocol_statistics: Vec<ProtocolStatistic>,
}

impl Default for Network {
    fn default() -> Network {
        Network {
            enabled: default_enabled(),
            interface_statistics: default_interface_statistics(),
            protocol_statistics: default_protocol_statistics(),
        }
    }
}

fn default_enabled() -> AtomicBool {
    AtomicBool::new(false)
}

fn default_interface_statistics() -> Vec<InterfaceStatistic> {
    vec![
        InterfaceStatistic::RxBytes,
        InterfaceStatistic::RxCrcErrors,
        InterfaceStatistic::RxDropped,
        InterfaceStatistic::RxDiscardsPhy,
        InterfaceStatistic::RxErrors,
        InterfaceStatistic::RxFifoErrors,
        InterfaceStatistic::RxMissedErrors,
        InterfaceStatistic::RxPackets,
        InterfaceStatistic::TxBytes,
        InterfaceStatistic::TxDiscardsPhy,
        InterfaceStatistic::TxDropped,
        InterfaceStatistic::TxErrors,
        InterfaceStatistic::TxFifoErrors,
        InterfaceStatistic::TxPackets,
    ]
}

fn default_protocol_statistics() -> Vec<ProtocolStatistic> {
    vec![
        ProtocolStatistic::TcpInSegs,
        ProtocolStatistic::TcpOutSegs,
        ProtocolStatistic::TcpPruneCalled,
        ProtocolStatistic::TcpRetransSegs,
        ProtocolStatistic::UdpInDatagrams,
        ProtocolStatistic::UdpInErrors,
        ProtocolStatistic::UdpOutDatagrams,
    ]
}

impl Network {
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn interface_statistics(&self) -> &[InterfaceStatistic] {
        &self.interface_statistics
    }

    pub fn protocol_statistics(&self) -> &[ProtocolStatistic] {
        &self.protocol_statistics
    }
}
