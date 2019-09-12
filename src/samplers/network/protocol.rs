// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::network::statistics::ProtocolStatistic;

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

    pub fn get(&self, statistic: &ProtocolStatistic) -> Option<&u64> {
        if let Some(inner) = self.data.get(statistic.protocol()) {
            inner.get(statistic.name())
        } else {
            None
        }
    }
}
