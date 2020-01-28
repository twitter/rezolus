// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use metrics::{Source, Statistic};
#[cfg(feature = "perf")]
pub use perfcnt::linux::*;
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{EnumString, IntoStaticStr};

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq, Hash, Serialize,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum SchedulerStatistic {
    #[strum(serialize = "scheduler/cpu_migrations")]
    CpuMigrations,
    #[strum(serialize = "scheduler/runqueue/latency")]
    RunqueueLatency,
    #[strum(serialize = "scheduler/context_switches")]
    ContextSwitches,
    #[strum(serialize = "scheduler/processes/created")]
    ProcessesCreated,
    #[strum(serialize = "scheduler/processes/running")]
    ProcessesRunning,
    #[strum(serialize = "scheduler/processes/blocked")]
    ProcessesBlocked,
}

impl SchedulerStatistic {
    #[allow(dead_code)]
    pub fn ebpf_table(self) -> Option<&'static str> {
        match self {
            Self::RunqueueLatency => Some("runqueue_latency"),
            _ => None,
        }
    }

    #[cfg(feature = "perf")]
    pub fn perf_counter_builder(self) -> Option<PerfCounterBuilderLinux> {
        match self {
            Self::CpuMigrations => Some(PerfCounterBuilderLinux::from_software_event(
                SoftwareEventType::CpuMigrations,
            )),
            _ => None,
        }
    }
}

impl Statistic for SchedulerStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        match *self {
            Self::RunqueueLatency => Source::Distribution,
            Self::ProcessesRunning | Self::ProcessesBlocked => Source::Gauge,
            _ => Source::Counter,
        }
    }
}

impl TryFrom<&str> for SchedulerStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        SchedulerStatistic::from_str(s)
    }
}
