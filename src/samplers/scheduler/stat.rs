// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::SECOND;

use core::convert::TryFrom;
use core::str::FromStr;

#[cfg(feature = "bpf")]
use bcc::perf_event::*;
use rustcommon_metrics::*;
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

#[cfg(feature = "bpf")]
use crate::common::bpf::*;

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

    #[allow(dead_code)]
    pub fn perf_table(self) -> Option<&'static str> {
        match self {
            Self::CpuMigrations => Some("cpu_migrations"),
            _ => None,
        }
    }

    #[cfg(feature = "bpf")]
    pub fn bpf_probes_required(self) -> Vec<FunctionProbe> {
        // define the unique probes below.
        let finish_task_probe = FunctionProbe {
            name: String::from("finish_task_switch"),
            handler: String::from("trace_run"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let wakeup_probe = FunctionProbe {
            name: String::from("ttwu_do_wakeup"),
            handler: String::from("trace_ttwu_do_wakeup"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let new_task_probe = FunctionProbe {
            name: String::from("wake_up_new_task"),
            handler: String::from("trace_wake_up_new_task"),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };

        match self {
            Self::RunqueueLatency => [finish_task_probe, wakeup_probe, new_task_probe].to_vec(),
            _ => Vec::new(),
        }
    }

    #[cfg(feature = "bpf")]
    pub fn event(self) -> Option<Event> {
        match self {
            Self::CpuMigrations => Some(Event::Software(SoftwareEvent::CpuMigrations)),
            _ => None,
        }
    }

    pub fn max(&self) -> u64 {
        match self {
            Self::RunqueueLatency => SECOND,
            _ => 1_000_000_000,
        }
    }
}

impl Statistic<AtomicU64, AtomicU32> for SchedulerStatistic {
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
