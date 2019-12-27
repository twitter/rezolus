// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::fmt;
use metrics::Statistic;
use serde_derive::*;
use std::error::Error;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum CpuidleStatistic {
    Time(CState),
}

#[derive(Debug)]
pub struct ParseCStateError;

impl fmt::Display for ParseCStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error parsing cstate")
    }
}

impl Error for ParseCStateError {
    fn description(&self) -> &str {
        "Error parsing cstate"
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
pub enum CState {
    C0,
    C1,
    C1E,
    C2,
    C3,
    C6,
    C7,
    C8,
}

impl FromStr for CState {
    type Err = ParseCStateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "POLL" | "C0" => Ok(CState::C0),
            "C1" => Ok(CState::C1),
            "C1E" => Ok(CState::C1E),
            "C2" => Ok(CState::C2),
            "C3" => Ok(CState::C3),
            "C6" => Ok(CState::C6),
            "C7" => Ok(CState::C7),
            "C8" => Ok(CState::C8),
            _ => Err(ParseCStateError),
        }
    }
}

impl Statistic for CpuidleStatistic {
    fn name(&self) -> &str {
        match self {
            Self::Time(cstate) => match cstate {
                CState::C0 => "cpuidle/time/c0",
                CState::C1 => "cpuidle/time/c1",
                CState::C1E => "cpuidle/time/c1e",
                CState::C2 => "cpuidle/time/c2",
                CState::C3 => "cpuidle/time/c3",
                CState::C6 => "cpuidle/time/c6",
                CState::C7 => "cpuidle/time/c7",
                CState::C8 => "cpuidle/time/c8",
            },
        }
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Counter
    }
}
