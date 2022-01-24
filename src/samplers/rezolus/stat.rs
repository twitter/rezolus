// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::metrics::*;
use serde_derive::{Deserialize, Serialize};
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
pub enum RezolusStatistic {
    #[strum(serialize = "rezolus/cpu/user")]
    CpuUser,
    #[strum(serialize = "rezolus/cpu/system")]
    CpuSystem,
    #[strum(serialize = "rezolus/memory/virtual")]
    MemoryVirtual,
    #[strum(serialize = "rezolus/memory/resident")]
    MemoryResident,
}

impl Statistic for RezolusStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        match self {
            Self::MemoryVirtual | Self::MemoryResident => Source::Gauge,
            _ => Source::Counter,
        }
    }
}
