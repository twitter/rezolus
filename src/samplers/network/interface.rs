// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::net;
use crate::samplers::network::statistics::InterfaceStatistic;

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

    pub fn get_statistic(&self, statistic: InterfaceStatistic) -> Result<u64, ()> {
        if let Some(name) = &self.name {
            net::read_network_stat(&name, statistic.name())
        } else {
            Err(())
        }
    }
}
