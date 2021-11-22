// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use rustcommon_metrics::*;
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

#[cfg(feature = "bpf")]
use crate::common::bpf::*;

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

    #[cfg(feature = "bpf")]
    pub fn bpf_probes_required(self) -> Vec<Probe> {
        // define the unique probes below.
        let page_accessed_probe = Probe {
            name: "mark_page_accessed".to_string(),
            handler: "trace_mark_page_accessed".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let buffer_dirty_probe = Probe {
            name: "mark_buffer_dirty".to_string(),
            handler: "trace_mark_buffer_dirty".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let page_cache_lru_probe = Probe {
            name: "add_to_page_cache_lru".to_string(),
            handler: "trace_add_to_page_cache_lru".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };
        let page_dirtied_probe = Probe {
            name: "account_page_dirtied".to_string(),
            handler: "trace_account_page_dirtied".to_string(),
            probe_type: ProbeType::Kernel,
            probe_location: ProbeLocation::Entry,
            binary_path: None,
            sub_system: None,
        };

        match self {
            Self::Hit | Self::Miss => vec![
                page_accessed_probe,
                buffer_dirty_probe,
                page_cache_lru_probe,
                page_dirtied_probe,
            ],
        }
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
