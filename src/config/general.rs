// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_atomics::*;

use crate::config::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct General {
    listen: Option<String>,
    #[serde(with = "LevelDef")]
    #[serde(default = "default_logging_level")]
    logging: Level,
    #[serde(default = "default_interval")]
    interval: AtomicUsize,
    #[serde(default = "default_threads")]
    threads: usize,
    #[serde(default = "default_window")]
    window: AtomicUsize,
    #[serde(default = "default_fault_tolerant")]
    fault_tolerant: AtomicBool,
    #[serde(default = "default_reading_suffix")]
    reading_suffix: String,
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

    /// interval in ms between samples if no sampler specific interval
    pub fn interval(&self) -> usize {
        self.interval.load(Ordering::Relaxed)
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    /// windows for histogram lookback
    pub fn window(&self) -> usize {
        self.window.load(Ordering::Relaxed) as usize
    }

    pub fn fault_tolerant(&self) -> bool {
        self.fault_tolerant.load(Ordering::Relaxed)
    }

    pub fn reading_suffix(&self) -> Option<&str> {
        if self.reading_suffix.len() == 0 {
            None
        } else {
            Some(&self.reading_suffix)
        }
    }
}

impl Default for General {
    fn default() -> General {
        General {
            listen: None,
            logging: default_logging_level(),
            interval: default_interval(),
            threads: default_threads(),
            window: default_window(),
            fault_tolerant: default_fault_tolerant(),
            reading_suffix: default_reading_suffix(),
        }
    }
}

fn default_interval() -> AtomicUsize {
    AtomicUsize::new(1000)
}

fn default_threads() -> usize {
    1
}

fn default_window() -> AtomicUsize {
    AtomicUsize::new(60)
}

fn default_fault_tolerant() -> AtomicBool {
    AtomicBool::new(true)
}

fn default_reading_suffix() -> String {
    "count".to_string()
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
