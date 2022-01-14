// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;
use rustcommon_atomics::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
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
            hosts: Default::default(),
            topic: Default::default(),
        }
    }
}

fn default_enabled() -> AtomicBool {
    AtomicBool::new(false)
}

fn default_interval() -> AtomicUsize {
    AtomicUsize::new(500)
}

#[cfg(feature = "push_kafka")]
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
