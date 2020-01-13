// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::fmt;
use metrics::Statistic;
use serde_derive::*;
use std::error::Error;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash, Serialize)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
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

impl TryFrom<&str> for CpuidleStatistic {
    type Error = CpuidleStatisticParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        CpuidleStatistic::from_str(s)
    }
}

impl FromStr for CpuidleStatistic {
    type Err = CpuidleStatisticParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let stat = match s {
            "cpuidle/time/c0" => Self::Time(CState::C0),
            "cpuidle/time/c1" => Self::Time(CState::C1),
            "cpuidle/time/c1e" => Self::Time(CState::C1E),
            "cpuidle/time/c2" => Self::Time(CState::C2),
            "cpuidle/time/c3" => Self::Time(CState::C3),
            "cpuidle/time/c6" => Self::Time(CState::C6),
            "cpuidle/time/c7" => Self::Time(CState::C7),
            "cpuidle/time/c8" => Self::Time(CState::C8),
            _ => return Err(CpuidleStatisticParseError),
        };

        Ok(stat)
    }
}

impl Into<&str> for CpuidleStatistic {
    fn into(self) -> &'static str {
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
}

#[derive(Debug)]
pub struct CpuidleStatisticParseError;

impl std::fmt::Display for CpuidleStatisticParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid cpuidle statistic")
    }
}

impl std::error::Error for CpuidleStatisticParseError {
    fn description(&self) -> &str {
        "Error parsing cpuidle statistic"
    }
}

impl Statistic for CpuidleStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Counter
    }
}
