// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct General {
    listen: Option<String>,
    #[serde(with = "LevelDef")]
    #[serde(default = "default_logging_level")]
    logging: Level,
    memcache: Option<String>,
    #[serde(default = "default_interval")]
    interval: usize,
    #[serde(default = "default_sample_rate")]
    sample_rate: f64,
    #[serde(default = "default_sampler_timeout")]
    sampler_timeout: usize,
    #[serde(default = "default_sampler_max_timeouts")]
    sampler_max_timeouts: usize,
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

    pub fn interval(&self) -> Duration {
        std::time::Duration::new(self.interval as u64, 0)
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    pub fn sampler_timeout(&self) -> Duration {
        std::time::Duration::new(0, self.sampler_timeout as u32)
    }

    pub fn sampler_max_timeouts(&self) -> usize {
        self.sampler_max_timeouts
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
            sample_rate: default_sample_rate(),
            sampler_timeout: default_sampler_timeout(),
            sampler_max_timeouts: default_sampler_max_timeouts(),
            stats_log: None,
            memcache: None,
        }
    }
}

fn default_interval() -> usize {
    60
}

fn default_sample_rate() -> f64 {
    1.0
}

fn default_sampler_timeout() -> usize {
    50
}

fn default_sampler_max_timeouts() -> usize {
    5
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
