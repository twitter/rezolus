// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cpu {
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default = "default_statistics")]
    statistics: Vec<String>,
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            enabled: default_enabled(),
            statistics: default_statistics(),
        }
    }
}

fn default_enabled() -> bool {
    false
}

fn default_statistics() -> Vec<String> {
    vec!["user".to_string(), "system".to_string(), "idle".to_string()]
}

impl Cpu {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn statistics(&self) -> Vec<String> {
        self.statistics.clone()
    }
}
