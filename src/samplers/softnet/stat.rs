// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use num_derive::FromPrimitive;
use rustcommon_metrics::*;
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
    FromPrimitive,
    IntoStaticStr,
    PartialEq,
    Hash,
    Serialize,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum SoftnetStatistic {
    #[strum(serialize = "softnet/processed")]
    Processed = 0,
    #[strum(serialize = "softnet/dropped")]
    Dropped = 1,
    #[strum(serialize = "softnet/time_squeezed")]
    TimeSqueezed = 2,
    #[strum(serialize = "softnet/cpu_collision")]
    CpuCollision = 3,
    #[strum(serialize = "softnet/received_rps")]
    ReceivedRps = 4,
    #[strum(serialize = "softnet/flow_limit_count")]
    FlowLimitCount = 5,
}

impl Statistic<AtomicU64, AtomicU32> for SoftnetStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        Source::Counter
    }
}

impl TryFrom<&str> for SoftnetStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        SoftnetStatistic::from_str(s)
    }
}
