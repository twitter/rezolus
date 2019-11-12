// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde_derive::*;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum Statistic {
    BandwidthDiscard,
    BandwidthRead,
    BandwidthWrite,
    OperationsDiscard,
    OperationsRead,
    OperationsWrite,
}

impl std::fmt::Display for Statistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Statistic::BandwidthDiscard => write!(f, "disk/bandwidth/discard"),
            Statistic::BandwidthRead => write!(f, "disk/bandwidth/read"),
            Statistic::BandwidthWrite => write!(f, "disk/bandwidth/write"),
            Statistic::OperationsDiscard => write!(f, "disk/operations/discard"),
            Statistic::OperationsRead => write!(f, "disk/operations/read"),
            Statistic::OperationsWrite => write!(f, "disk/operations/write"),
        }
    }
}
