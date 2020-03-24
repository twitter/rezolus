// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use metrics::Statistic;
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

impl Statistic for SoftnetStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn description(&self) -> Option<&str> {
        Some(match self {
            Self::Processed => "total number of frames processed",
            Self::Dropped => "number of frames dropped due to no room on processing queue",
            Self::TimeSqueezed => "number of times net_rx_action had more work, but budget or time exhausted",
            Self::CpuCollision => "number of times collision occurred on obtaining device lock while transmitting",
            Self::ReceivedRps => "number of times CPU has been woken up to process packets via inter-processor interrupt",
            Self::FlowLimitCount => "number of times the flow limit has been reached",
        })
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Counter
    }
}

impl TryFrom<&str> for SoftnetStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        SoftnetStatistic::from_str(s)
    }
}
