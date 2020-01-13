// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use metrics::Statistic;
use serde_derive::*;
use std::str::FromStr;
use strum::ParseError;
use strum_macros::*;

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq, Hash, Serialize,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum SchedulerStatistic {
    #[strum(serialize = "scheduler/runqueue/latency")]
    RunqueueLatency,
}

impl SchedulerStatistic {
    #[allow(dead_code)]
    pub fn ebpf_table(self) -> Option<&'static str> {
        match self {
            Self::RunqueueLatency => Some("runqueue_latency"),
        }
    }
}

impl Statistic for SchedulerStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Distribution
    }
}

impl TryFrom<&str> for SchedulerStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        SchedulerStatistic::from_str(s)
    }
}
