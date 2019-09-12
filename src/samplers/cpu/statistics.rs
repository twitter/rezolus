// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde_derive::*;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Statistic {
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

impl std::fmt::Display for Statistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "cpu/user"),
            Self::Nice => write!(f, "cpu/nice"),
            Self::System => write!(f, "cpu/system"),
            Self::Idle => write!(f, "cpu/idle"),
            Self::Irq => write!(f, "cpu/irq"),
            Self::Softirq => write!(f, "cpu/softirq"),
            Self::Steal => write!(f, "cpu/steal"),
            Self::Guest => write!(f, "cpu/guest"),
            Self::GuestNice => write!(f, "cpu/guest_nice"),
        }
    }
}
