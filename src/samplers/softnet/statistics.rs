// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum Statistic {
    Processed,
    Dropped,
    TimeSqueezed,
    CpuCollision,
    ReceivedRps,
    FlowLimitCount,
}

impl std::fmt::Display for Statistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Statistic::Processed => write!(f, "softnet/processed"),
            Statistic::Dropped => write!(f, "softnet/dropped"),
            Statistic::TimeSqueezed => write!(f, "softnet/time_squeezed"),
            Statistic::CpuCollision => write!(f, "softnet/cpu_collision"),
            Statistic::ReceivedRps => write!(f, "softnet/received_rps"),
            Statistic::FlowLimitCount => write!(f, "softnet/flow_limit_count"),
        }
    }
}

impl Statistic {
    pub fn field_number(&self) -> usize {
        match self {
            Statistic::Processed => 0,
            Statistic::Dropped => 1,
            Statistic::TimeSqueezed => 2,
            Statistic::CpuCollision => 3,
            Statistic::ReceivedRps => 4,
            Statistic::FlowLimitCount => 5,
        }
    }
}