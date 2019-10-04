// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;
use crate::samplers::cpu::statistics::*;

use atomics::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cpu {
    #[serde(default = "default_enabled")]
    enabled: AtomicBool,
    #[serde(default = "default_interval")]
    interval: AtomicOption<AtomicUsize>,
    #[serde(default = "default_statistics")]
    statistics: Vec<Statistic>,
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            enabled: default_enabled(),
            interval: default_interval(),
            statistics: default_statistics(),
        }
    }
}

fn default_enabled() -> AtomicBool {
    AtomicBool::new(false)
}

fn default_interval() -> AtomicOption<AtomicUsize> {
    AtomicOption::none()
}

fn default_statistics() -> Vec<Statistic> {
    vec![Statistic::User, Statistic::System, Statistic::Idle]
}

impl SamplerConfig for Cpu {
    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    fn interval(&self) -> Option<usize> {
        self.interval.load(Ordering::Relaxed)
    }
}

impl Cpu {
    pub fn statistics(&self) -> Vec<Statistic> {
        self.statistics.clone()
    }
}
