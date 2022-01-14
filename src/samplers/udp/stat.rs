// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::metrics::*;
use serde_derive::{Deserialize, Serialize};
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

impl Statistic<AtomicU64, AtomicU32> for UdpStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        Source::Counter
    }
}
