// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

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
pub enum InterruptStatistic {
    #[strum(serialize = "interrupt/total")]
    Total,
    #[strum(serialize = "interrupt/timer")]
    Timer,
    #[strum(serialize = "interrupt/nmi")]
    NonMaskable,
    #[strum(serialize = "interrupt/nvme")]
    Nvme,
    #[strum(serialize = "interrupt/network")]
    Network,
    #[strum(serialize = "interrupt/local_timer")]
    LocalTimer,
    #[strum(serialize = "interrupt/spurious")]
    Spurious,
    #[strum(serialize = "interrupt/performance_monitoring")]
    PerformanceMonitoring,
    #[strum(serialize = "interrupt/rescheduling")]
    Rescheduling,
    #[strum(serialize = "interrupt/function_call")]
    FunctionCall,
    #[strum(serialize = "interrupt/tlb_shootdowns")]
    TlbShootdowns,
    #[strum(serialize = "interrupt/thermal_event")]
    ThermalEvent,
    #[strum(serialize = "interrupt/machine_check_exception")]
    MachineCheckException,
    #[strum(serialize = "interrupt/rtc")]
    RealTimeClock,
    #[strum(serialize = "interrupt/node0/total")]
    Node0Total,
    #[strum(serialize = "interrupt/node1/total")]
    Node1Total,
    #[strum(serialize = "interrupt/node0/network")]
    Node0Network,
    #[strum(serialize = "interrupt/node1/network")]
    Node1Network,
    #[strum(serialize = "interrupt/node0/nvme")]
    Node0Nvme,
    #[strum(serialize = "interrupt/node1/nvme")]
    Node1Nvme,
    #[strum(serialize = "interrupt/softirq/hi")]
    SoftIrqHI,
    #[strum(serialize = "interrupt/softirq/timer")]
    SoftIrqTimer,
    #[strum(serialize = "interrupt/softirq/net_rx")]
    SoftIrqNetRx,
    #[strum(serialize = "interrupt/softirq/net_tx")]
    SoftIrqNetTx,
    #[strum(serialize = "interrupt/softirq/block")]
    SoftIrqBlock,
    #[strum(serialize = "interrupt/softirq/irq_poll")]
    SoftIrqPoll,
    #[strum(serialize = "interrupt/softirq/tasklet")]
    SoftIrqTasklet,
    #[strum(serialize = "interrupt/softirq/sched")]
    SoftIrqSched,
    #[strum(serialize = "interrupt/softirq/hr_timer")]
    SoftIrqHRTimer,
    #[strum(serialize = "interrupt/softirq/rcu")]
    SoftIrqRCU,
    #[strum(serialize = "interrupt/softirq/unknown")]
    SoftIrqUnknown,
    #[strum(serialize = "interrupt/hardirq")]
    HardIrq,
}

impl InterruptStatistic {
    pub fn bpf_table(self) -> Option<&'static str> {
        match self {
            Self::SoftIrqHI => Some("hi"),
            Self::SoftIrqTimer => Some("timer"),
            Self::SoftIrqNetRx => Some("net_rx"),
            Self::SoftIrqNetTx => Some("net_tx"),
            Self::SoftIrqBlock => Some("block"),
            Self::SoftIrqPoll => Some("irq_poll"),
            Self::SoftIrqTasklet => Some("tasklet"),
            Self::SoftIrqSched => Some("sched"),
            Self::SoftIrqHRTimer => Some("hr_timer"),
            Self::SoftIrqRCU => Some("rcu"),
            Self::SoftIrqUnknown => Some("unknown"),
            Self::HardIrq => Some("hardirq_total"),
            _ => None,
        }
    }

    #[cfg(feature = "bpf")]
    pub fn bpf_probes_required(self) -> Vec<Probe> {
        // define the unique probes below.
        let irq_event_percpu_probe = Probe {
            name: "handle_irq_event_percpu".to_string(),
            handler: "hardirq_entry".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let irq_event_percpu_ret_probe = Probe {
            name: "handle_irq_event_percpu".to_string(),
            handler: "hardirq_exit".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Return,
            binary_path: None,
            sub_system: None,
        };
        let softirq_entry_tracepoint = Probe {
            name: "softirq_entry".to_string(),
            handler: "softirq_entry".to_string(),
            probe_type: ProbeType::Tracepoint,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: Some("irq".to_string()),
        };
        let softirq_exit_tracepoint = Probe {
            name: "softirq_exit".to_string(),
            handler: "softirq_exit".to_string(),
            probe_type: ProbeType::Tracepoint,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: Some("irq".to_string()),
        };

        // specify what probes are required for each telemetry.
        match self {
            Self::SoftIrqHI
            | Self::SoftIrqTimer
            | Self::SoftIrqNetRx
            | Self::SoftIrqNetTx
            | Self::SoftIrqBlock
            | Self::SoftIrqPoll
            | Self::SoftIrqTasklet
            | Self::SoftIrqSched
            | Self::SoftIrqHRTimer
            | Self::SoftIrqRCU
            | Self::SoftIrqUnknown => vec![softirq_entry_tracepoint, softirq_exit_tracepoint],
            Self::HardIrq => vec![irq_event_percpu_probe, irq_event_percpu_ret_probe],
            _ => Vec::new(),
        }
    }
}

impl TryFrom<&str> for InterruptStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        InterruptStatistic::from_str(s)
    }
}

impl Statistic<AtomicU64, AtomicU32> for InterruptStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        if self.bpf_table().is_some() {
            Source::Distribution
        } else {
            Source::Counter
        }
    }
}
