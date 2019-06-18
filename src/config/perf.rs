// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Perf {
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default = "default_events")]
    events: Vec<String>,
}

impl Default for Perf {
    fn default() -> Perf {
        Perf {
            enabled: default_enabled(),
            events: default_events(),
        }
    }
}

fn default_enabled() -> bool {
    false
}

fn default_events() -> Vec<String> {
    vec![
        "CacheMisses".to_string(),
        "CacheReferences".to_string(),
        "ContextSwitches".to_string(),
        "CpuBranchInstructions".to_string(),
        "CpuBranchMisses".to_string(),
        "CpuCycles".to_string(),
        "CpuInstructions".to_string(),
        "CpuMigrations".to_string(),
        "CpuRefCycles".to_string(),
        "DtlbLoads".to_string(),
        "DtlbLoadMisses".to_string(),
        "DtlbStores".to_string(),
        "DtlbStoreMisses".to_string(),
        "MemoryLoads".to_string(),
        "MemoryLoadMisses".to_string(),
        "MemoryStores".to_string(),
        "MemoryStoreMisses".to_string(),
        "PageFaults".to_string(),
        "StalledCyclesBackend".to_string(),
        "StalledCyclesFrontend".to_string(),
    ]
}

impl Perf {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn events(&self) -> &[String] {
        &self.events
    }
}
