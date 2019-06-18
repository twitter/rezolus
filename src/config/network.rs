// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default = "default_interface_statistics")]
    interface_statistics: Vec<String>,
    #[serde(default = "default_protocol_statistics")]
    protocol_statistics: Vec<String>,
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

fn default_enabled() -> bool {
    false
}

fn default_interface_statistics() -> Vec<String> {
    vec![
        "rx_bytes".to_string(),
        "rx_crc_errors".to_string(),
        "rx_dropped".to_string(),
        "rx_discards_phy".to_string(),
        "rx_errors".to_string(),
        "rx_fifo_errors".to_string(),
        "rx_missed_errors".to_string(),
        "rx_packets".to_string(),
        "tx_bytes".to_string(),
        "tx_discards_phy".to_string(),
        "tx_dropped".to_string(),
        "tx_errors".to_string(),
        "tx_fifo_errors".to_string(),
        "tx_packets".to_string(),
    ]
}

fn default_protocol_statistics() -> Vec<String> {
    vec![
        "TcpInSegs".to_string(),
        "TcpOutSegs".to_string(),
        "TcpPruneCalled".to_string(),
        "TcpRetransSegs".to_string(),
        "UdpInDatagrams".to_string(),
        "UdpInErrors".to_string(),
        "UdpOutDatagrams".to_string(),
    ]
}

impl Network {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn interface_statistics(&self) -> &[String] {
        &self.interface_statistics
    }

    pub fn protocol_statistics(&self) -> &[String] {
        &self.protocol_statistics
    }
}
