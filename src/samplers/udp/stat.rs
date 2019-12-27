// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum UdpStatistic {
    InDatagrams,
    InErrors,
    OutDatagrams,
}

impl UdpStatistic {
    pub fn keys(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::InDatagrams => Some(("Udp:", "InDatagrams")),
            Self::InErrors => Some(("Udp:", "InErrors")),
            Self::OutDatagrams => Some(("Udp:", "OutDatagrams")),
        }
    }
}

impl Statistic for UdpStatistic {
    fn name(&self) -> &str {
        match self {
            Self::InDatagrams => "udp/receive/datagrams",
            Self::InErrors => "udp/receive/errors",
            Self::OutDatagrams => "udp/transmit/datagrams",
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::InDatagrams => Some("udp datagrams received"),
            Self::InErrors => Some("udp datagrams that were not delivered to valid port"),
            Self::OutDatagrams => Some("udp datagrams transmitted"),
        }
    }

    fn unit(&self) -> Option<&str> {
        match self {
            Self::InDatagrams | Self::OutDatagrams => Some("datagrams"),
            _ => None,
        }
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Counter
    }
}
