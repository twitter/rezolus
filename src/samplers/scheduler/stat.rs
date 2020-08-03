// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::SECOND;
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
    pub fn bpf_table(self) -> Option<&'static str> {
        match self {
            Self::RunqueueLatency => Some("runqueue_latency"),
            _ => None,
        }
    }

    #[cfg(feature = "perf")]
    pub fn perf_config(self) -> Option<(&'static str, bcc::perf::Event)> {
        use bcc::perf::{Event, SoftwareEvent};
        match self {
            Self::CpuMigrations => Some((
                "cpu_migrations",
                Event::Software(SoftwareEvent::CpuMigrations),
            )),
            _ => None,
        }
    }

    pub fn max(&self) -> u64 {
        if self.bpf_table().is_some() {
            SECOND
        } else if *self == SchedulerStatistic::ContextSwitches {
            1_000_000_000
        } else {
            1_000_000
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
