// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use rustcommon_metrics::{Source, Statistic};
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

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

    fn source(&self) -> Source {
        Source::Counter
    }
}

impl TryFrom<&str> for UdpStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        UdpStatistic::from_str(s)
    }
}
