// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
use serde_derive::*;
use std::convert::TryFrom;
use std::str::FromStr;
use strum::ParseError;
use strum_macros::*;

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq, Hash, Serialize,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum UdpStatistic {
    #[strum(serialize = "udp/receive/datagrams")]
    InDatagrams,
    #[strum(serialize = "udp/receive/errors")]
    InErrors,
    #[strum(serialize = "udp/transmit/datagrams")]
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
        (*self).into()
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

impl TryFrom<&str> for UdpStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        UdpStatistic::from_str(s)
    }
}
