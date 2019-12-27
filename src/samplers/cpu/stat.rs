// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum CpuStatistic {
    User,
    Nice,
    System,
    Idle,
    Irq,
    Softirq,
    Steal,
    Guest,
    GuestNice,
}

impl Statistic for CpuStatistic {
    fn name(&self) -> &str {
        match self {
            Self::User => "cpu/user",
            Self::Nice => "cpu/nice",
            Self::System => "cpu/system",
            Self::Idle => "cpu/idle",
            Self::Irq => "cpu/irq",
            Self::Softirq => "cpu/softirq",
            Self::Steal => "cpu/steal",
            Self::Guest => "cpu/guest",
            Self::GuestNice => "cpu/guestnice",
        }
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Counter
    }
}
