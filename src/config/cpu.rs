// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;
use crate::samplers::cpu::CpuStatistic;
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cpu {
    #[serde(default = "default_enabled")]
    enabled: AtomicBool,
    #[serde(default = "default_statistics")]
    statistics: Vec<CpuStatistic>,
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            enabled: default_enabled(),
            statistics: default_statistics(),
        }
    }
}

fn default_enabled() -> AtomicBool {
    AtomicBool::new(false)
}

fn default_statistics() -> Vec<CpuStatistic> {
    vec![CpuStatistic::User, CpuStatistic::System, CpuStatistic::Idle]
}

impl Cpu {
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn statistics(&self) -> Vec<CpuStatistic> {
        self.statistics.clone()
    }
}
