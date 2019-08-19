// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::samplers::Statistic;
use std::collections::HashMap;
use std::fmt;

use serde_derive::*;

#[derive(Clone, Debug)]
pub struct Protocol {
    data: HashMap<String, HashMap<String, u64>>,
}

impl Protocol {
    pub fn new() -> Result<Self, ()> {
        let snmp = crate::common::file::nested_map_from_file("/proc/net/snmp")?;
        let netstat = crate::common::file::nested_map_from_file("/proc/net/netstat")?;
        let data = snmp.into_iter().chain(netstat).collect();
        Ok(Self { data })
    }

    pub fn get(&self, statistic: &ProtocolStatistic) -> Option<&u64> {
        if let Some(inner) = self.data.get(statistic.protocol()) {
            inner.get(statistic.name())
        } else {
            None
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

impl fmt::Display for ProtocolStatistic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProtocolStatistic::TcpInSegs => write!(f, "network/tcp/receive/segments"),
            ProtocolStatistic::TcpOutSegs => write!(f, "network/tcp/transmit/segments"),
            ProtocolStatistic::TcpPruneCalled => write!(f, "network/tcp/receive/prune_called"),
            ProtocolStatistic::TcpRcvCollapsed => write!(f, "network/tcp/receive/collapsed"),
            ProtocolStatistic::TcpRetransSegs => write!(f, "network/tcp/transmit/retranmits"),
            ProtocolStatistic::UdpInDatagrams => write!(f, "network/udp/receive/datagrams"),
            ProtocolStatistic::UdpInErrors => write!(f, "network/udp/receive/errors"),
            ProtocolStatistic::UdpOutDatagrams => write!(f, "network/udp/transmit/datagrams"),
        }
    }
}

impl Statistic for ProtocolStatistic {}

impl ProtocolStatistic {
    pub fn name(&self) -> &str {
        match self {
            ProtocolStatistic::TcpInSegs => "InSegs",
            ProtocolStatistic::TcpOutSegs => "OutSegs",
            ProtocolStatistic::TcpPruneCalled => "PruneCalled",
            ProtocolStatistic::TcpRcvCollapsed => "TCPRcvCollapsed",
            ProtocolStatistic::TcpRetransSegs => "RetransSegs",
            ProtocolStatistic::UdpInDatagrams => "InDatagrams",
            ProtocolStatistic::UdpInErrors => "InErrors",
            ProtocolStatistic::UdpOutDatagrams => "OutDatagrams",
        }
    }

    pub fn protocol(&self) -> &str {
        match self {
            ProtocolStatistic::TcpInSegs
            | ProtocolStatistic::TcpOutSegs
            | ProtocolStatistic::TcpRetransSegs => "Tcp:",
            ProtocolStatistic::TcpPruneCalled | ProtocolStatistic::TcpRcvCollapsed => "TcpExt:",
            ProtocolStatistic::UdpInDatagrams
            | ProtocolStatistic::UdpInErrors
            | ProtocolStatistic::UdpOutDatagrams => "Udp:",
        }
    }
}
