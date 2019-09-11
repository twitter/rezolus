// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum Statistic {
	CpuSystem,
	CpuTotal,
	CpuUser,
}

impl std::fmt::Display for Statistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
        	Self::CpuSystem => write!(f, "container/cpu/system"),
            Self::CpuTotal => write!(f, "container/cpu/total"),
            Self::CpuUser => write!(f, "container/cpu/user"),
        }
    }
}