// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use logger::*;

use std::collections::HashMap;

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

    pub fn get(&self, statistic: &Statistic) -> Option<&u64> {
        if let Some(inner) = self.data.get(statistic.protocol()) {
            inner.get(statistic.name())
        } else {
            None
        }
    }
}

pub enum Statistic {
    TcpInSegs,
    TcpOutSegs,
    TcpPruneCalled,
    TcpRcvCollapsed,
    TcpRetransSegs,
    UdpInDatagrams,
    UdpInErrors,
    UdpOutDatagrams,
}

impl Statistic {
    pub fn from_str(statistic: &str) -> Result<Statistic, ()> {
        match statistic {
            "TcpInSegs" => Ok(Statistic::TcpInSegs),
            "TcpOutSegs" => Ok(Statistic::TcpOutSegs),
            "TcpPruneCalled" => Ok(Statistic::TcpPruneCalled),
            "TcpRcvCollapsed" => Ok(Statistic::TcpRcvCollapsed),
            "TcpRetransSegs" => Ok(Statistic::TcpRetransSegs),
            "UdpInDatagrams" => Ok(Statistic::UdpInDatagrams),
            "UdpInErrors" => Ok(Statistic::UdpInErrors),
            "UdpOutDatagrams" => Ok(Statistic::UdpOutDatagrams),
            _ => {
                debug!("unknown statistic: {}", statistic);
                Err(())
            }
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Statistic::TcpInSegs => "network/tcp/receive/segments",
            Statistic::TcpOutSegs => "network/tcp/transmit/segments",
            Statistic::TcpPruneCalled => "network/tcp/receive/prune_called",
            Statistic::TcpRcvCollapsed => "network/tcp/receive/collapsed",
            Statistic::TcpRetransSegs => "network/tcp/transmit/retransmits",
            Statistic::UdpInDatagrams => "network/udp/receive/datagrams",
            Statistic::UdpInErrors => "network/udp/receive/errors",
            Statistic::UdpOutDatagrams => "network/udp/transmit/datagrams",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Statistic::TcpInSegs => "InSegs",
            Statistic::TcpOutSegs => "OutSegs",
            Statistic::TcpPruneCalled => "PruneCalled",
            Statistic::TcpRcvCollapsed => "TCPRcvCollapsed",
            Statistic::TcpRetransSegs => "RetransSegs",
            Statistic::UdpInDatagrams => "InDatagrams",
            Statistic::UdpInErrors => "InErrors",
            Statistic::UdpOutDatagrams => "OutDatagrams",
        }
    }

    pub fn protocol(&self) -> &str {
        match self {
            Statistic::TcpInSegs | Statistic::TcpOutSegs | Statistic::TcpRetransSegs => "Tcp:",
            Statistic::TcpPruneCalled | Statistic::TcpRcvCollapsed => "TcpExt:",
            Statistic::UdpInDatagrams | Statistic::UdpInErrors | Statistic::UdpOutDatagrams => {
                "Udp:"
            }
        }
    }
}
