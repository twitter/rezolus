// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::SECOND;

use crate::metrics::*;
#[cfg(feature = "bpf")]
use bcc::perf_event::*;
use serde_derive::{Deserialize, Serialize};
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
    pub fn bpf_probes_required(self) -> Vec<Probe> {
        // define the unique probes below.
        let finish_task_probe = Probe {
            name: "finish_task_switch".to_string(),
            handler: "trace_run".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let wakeup_probe = Probe {
            name: "ttwu_do_wakeup".to_string(),
            handler: "trace_ttwu_do_wakeup".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let new_task_probe = Probe {
            name: "wake_up_new_task".to_string(),
            handler: "trace_wake_up_new_task".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };

        match self {
            Self::RunqueueLatency => vec![finish_task_probe, wakeup_probe, new_task_probe],
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
