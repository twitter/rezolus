// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use rustcommon_metrics::{Source, Statistic};
#[cfg(feature = "perf")]
pub use perfcnt::linux::*;
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
        Source::Counter
    }
}
