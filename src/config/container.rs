// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use atomics::*;

use crate::config::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Container {
    #[serde(default = "default_enabled")]
    enabled: AtomicBool,
    #[serde(default = "default_interval")]
    interval: AtomicOption<AtomicUsize>,
}

impl Default for Container {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            interval: default_interval(),
        }
    }
}

fn default_enabled() -> AtomicBool {
    AtomicBool::new(false)
}

fn default_interval() -> AtomicOption<AtomicUsize> {
    AtomicOption::none()
}

impl SamplerConfig for Container {
    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    fn interval(&self) -> Option<usize> {
        self.interval.load(Ordering::Relaxed)
    }
}
