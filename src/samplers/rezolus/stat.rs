// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;

use core::convert::TryFrom;
use serde_derive::*;
use std::str::FromStr;
use strum::ParseError;
use strum_macros::*;

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq, Hash, Serialize,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum RezolusStatistic {
    #[strum(serialize = "rezolus/cpu/user")]
    UserTime,
    #[strum(serialize = "rezolus/cpu/system")]
    SystemTime,
    #[strum(serialize = "rezolus/memory/virtual")]
    VirtualMemory,
    #[strum(serialize = "rezolus/memory/resident")]
    ResidentMemory,
}

impl Statistic for RezolusStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> metrics::Source {
        match self {
            Self::VirtualMemory | Self::ResidentMemory => metrics::Source::Gauge,
            _ => metrics::Source::Counter,
        }
    }
}

impl TryFrom<&str> for RezolusStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        RezolusStatistic::from_str(s)
    }
}
