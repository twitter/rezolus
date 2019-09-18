// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum Statistic {
    MemoryVirtual,
    MemoryResident,
    CpuUser,
    CpuKernel,
}

impl std::fmt::Display for Statistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Statistic::MemoryVirtual => write!(f, "rezolus/memory/virtual"),
            Statistic::MemoryResident => write!(f, "rezolus/memory/resident"),
            Statistic::CpuUser => write!(f, "rezolus/cpu/user"),
            Statistic::CpuKernel => write!(f, "rezolus/cpu/kernel"),
        }
    }
}
