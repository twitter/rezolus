use rustcommon_metrics::{AtomicU32, AtomicU64, Source, Statistic};
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

use core::convert::TryFrom;
use core::str::FromStr;

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
pub enum LibCallStatistic {
    #[strum(serialize = "foo/bar")]
    FooBar,
}

impl LibCallStatistic {}

impl Statistic<AtomicU64, AtomicU32> for LibCallStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        Source::Counter
    }
}

impl TryFrom<&str> for LibCallStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        LibCallStatistic::from_str(s)
    }
}
