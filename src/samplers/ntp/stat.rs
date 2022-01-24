// Copyright 2019 Twitter, Inc.
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
pub enum NtpStatistic {
    #[strum(serialize = "ntp/estimated_error")]
    EstimatedError,
    #[strum(serialize = "ntp/maximum_error")]
    MaximumError,
}

impl Statistic for NtpStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        Source::Gauge
    }
}
