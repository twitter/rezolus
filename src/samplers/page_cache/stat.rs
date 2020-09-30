// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use rustcommon_metrics::*;
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
pub enum PageCacheStatistic {
    #[strum(serialize = "page_cache/hit")]
    Hit,
    #[strum(serialize = "page_cache/miss")]
    Miss,
}

impl PageCacheStatistic {
    pub fn is_bpf(&self) -> bool {
        true
    }
}

impl TryFrom<&str> for PageCacheStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        PageCacheStatistic::from_str(s)
    }
}

impl Statistic<AtomicU64, AtomicU32> for PageCacheStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        Source::Counter
    }
}
