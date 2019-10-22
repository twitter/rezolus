// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;
use crate::samplers::cpuidle::statistics::*;

use atomics::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CpuIdle {
    #[serde(default = "default_enabled")]
    enabled: AtomicBool,
    #[serde(default = "default_interval")]
    interval: AtomicOption<AtomicUsize>,
    #[serde(default = "default_statistics")]
    statistics: Vec<Statistic>,
}

impl Default for CpuIdle {
    fn default() -> CpuIdle {
        CpuIdle {
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
    vec![
        Statistic::State0,
        Statistic::State1,
        Statistic::State2,
        Statistic::State3,
    ]
}

impl SamplerConfig for CpuIdle {
    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    fn interval(&self) -> Option<usize> {
        self.interval.load(Ordering::Relaxed)
    }
}

impl CpuIdle {
    pub fn statistics(&self) -> Vec<Statistic> {
        self.statistics.clone()
    }
}
