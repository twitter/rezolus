// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;
use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct General {
    listen: Option<String>,
    #[serde(with = "LevelDef")]
    #[serde(default = "default_logging_level")]
    logging: Level,
    memcache: Option<String>,
    #[serde(default = "default_interval")]
    interval: AtomicUsize,
    #[serde(default = "default_window")]
    window: AtomicUsize,
    #[serde(default = "default_timeout")]
    timeout: AtomicUsize,
    #[serde(default = "default_max_timeouts")]
    max_timeouts: AtomicUsize,
    stats_log: Option<String>,
}

impl General {
    pub fn listen(&self) -> Option<String> {
        self.listen.clone()
    }

    pub fn logging(&self) -> Level {
        self.logging
    }

    pub fn set_logging(&mut self, level: Level) {
        self.logging = level;
    }

    pub fn interval(&self) -> usize {
        self.interval.load(Ordering::Relaxed)
    }

    pub fn window(&self) -> Duration {
        Duration::new(self.window.load(Ordering::Relaxed) as u64, 0)
    }

    pub fn timeout(&self) -> usize {
        self.timeout.load(Ordering::Relaxed)
    }

    pub fn max_timeouts(&self) -> usize {
        self.max_timeouts.load(Ordering::Relaxed)
    }

    pub fn memcache(&self) -> Option<String> {
        self.memcache.clone()
    }

    pub fn stats_log(&self) -> Option<String> {
        self.stats_log.clone()
    }
}

impl Default for General {
    fn default() -> General {
        General {
            listen: None,
            logging: default_logging_level(),
            interval: default_interval(),
            window: default_window(),
            timeout: default_timeout(),
            max_timeouts: default_max_timeouts(),
            stats_log: None,
            memcache: None,
        }
    }
}

fn default_interval() -> AtomicUsize {
    AtomicUsize::new(1000)
}

fn default_window() -> AtomicUsize {
    AtomicUsize::new(60)
}

fn default_timeout() -> AtomicUsize {
    AtomicUsize::new(50)
}

fn default_max_timeouts() -> AtomicUsize {
    AtomicUsize::new(5)
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[serde(remote = "Level")]
#[serde(deny_unknown_fields)]
enum LevelDef {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

fn default_logging_level() -> Level {
    Level::Info
}
