// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde_derive::*;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Statistic {
    State0,
    State1,
    State2,
    State3,
}

impl std::fmt::Display for Statistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::State0 => write!(f, "cpuidle/state0"),
            Self::State1 => write!(f, "cpuidle/state1"),
            Self::State2 => write!(f, "cpuidle/state2"),
            Self::State3 => write!(f, "cpuidle/state3"),
        }
    }
}
