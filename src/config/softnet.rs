// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;

use atomics::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Softnet {
    #[serde(default = "default_enabled")]
    enabled: AtomicBool,
}

impl Default for Softnet {
    fn default() -> Softnet {
        Softnet {
            enabled: default_enabled(),
        }
    }
}

fn default_enabled() -> AtomicBool {
    AtomicBool::new(false)
}

impl Softnet {
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}
