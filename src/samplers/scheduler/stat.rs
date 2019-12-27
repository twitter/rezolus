// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum SchedulerStatistic {
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
        match self {
            Self::RunqueueLatency => "scheduler/runqueue/latency",
        }
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Distribution
    }
}
