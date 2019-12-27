// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum SoftnetStatistic {
    Processed = 0,
    Dropped = 1,
    TimeSqueezed = 2,
    CpuCollision = 3,
    ReceivedRps = 4,
    FlowLimitCount = 5,
}

impl Statistic for SoftnetStatistic {
    fn name(&self) -> &str {
        match self {
            Self::Processed => "softnet/processed",
            Self::Dropped => "softnet/dropped",
            Self::TimeSqueezed => "softnet/time_squeezed",
            Self::CpuCollision => "softnet/cpu_collision",
            Self::ReceivedRps => "softnet/received_rps",
            Self::FlowLimitCount => "softnet/flow_limit_count",
        }
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Counter
    }
}
