// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;

use atomics::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kafka {
    #[serde(default = "default_enabled")]
    enabled: AtomicBool,
    #[serde(default = "default_interval")]
    interval: AtomicUsize,
    hosts: Vec<String>,
    topic: Option<String>,
}

impl Default for Kafka {
    fn default() -> Kafka {
        Kafka {
            enabled: default_enabled(),
            interval: default_interval(),
            hosts: vec![],
            topic: None,
        }
    }
}

fn default_enabled() -> AtomicBool {
    AtomicBool::new(false)
}

fn default_interval() -> AtomicUsize {
    AtomicUsize::new(500)
}

impl Kafka {
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn interval(&self) -> usize {
        self.interval.load(Ordering::Relaxed)
    }

    pub fn hosts(&self) -> Vec<String> {
        self.hosts.clone()
    }

    pub fn topic(&self) -> Option<String> {
        self.topic.clone()
    }
}
