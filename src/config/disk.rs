// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Disk {
    #[serde(default = "default_enabled")]
    enabled: bool,
}

impl Default for Disk {
    fn default() -> Disk {
        Disk {
            enabled: default_enabled(),
        }
    }
}

fn default_enabled() -> bool {
    false
}

impl Disk {
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}
