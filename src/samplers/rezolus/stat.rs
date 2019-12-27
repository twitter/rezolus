// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;

use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum RezolusStatistic {
    UserTime,
    SystemTime,
    VirtualMemory,
    ResidentMemory,
}

impl Statistic for RezolusStatistic {
    fn name(&self) -> &str {
        match self {
            Self::UserTime => "rezolus/cpu/user",
            Self::SystemTime => "rezolus/cpu/system",
            Self::VirtualMemory => "rezolus/memory/virtual",
            Self::ResidentMemory => "rezolus/memory/resident",
        }
    }

    fn source(&self) -> metrics::Source {
        match self {
            Self::VirtualMemory | Self::ResidentMemory => metrics::Source::Gauge,
            _ => metrics::Source::Counter,
        }
    }
}
