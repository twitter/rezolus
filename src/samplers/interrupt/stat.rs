// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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
    #[strum(serialize = "interrupt/serial")]
    Serial,
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
}

impl TryFrom<&str> for InterruptStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        InterruptStatistic::from_str(s)
    }
}

impl Statistic for InterruptStatistic {
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
