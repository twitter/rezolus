// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use atomics::AtomicU32;
use metrics::*;
use tokio::runtime::Handle;
use tokio::time::{interval, Interval};

use crate::config::General as GeneralConfig;
use crate::config::{Config, SamplerConfig};

pub mod cpu;
pub mod disk;
pub mod ext4;
pub mod memcache;
pub mod memory;
pub mod network;
pub mod rezolus;
pub mod scheduler;
pub mod softnet;
pub mod tcp;
pub mod udp;
pub mod xfs;

pub use cpu::Cpu;
pub use disk::Disk;
pub use ext4::Ext4;
pub use memory::Memory;
pub use network::Network;
pub use rezolus::Rezolus;
pub use scheduler::Scheduler;
pub use softnet::Softnet;
pub use tcp::Tcp;
pub use udp::Udp;
pub use xfs::Xfs;

#[async_trait]
pub trait Sampler: Sized + Send {
    type Statistic: Statistic;

    /// Create a new instance of the sampler
    fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Result<Self, failure::Error>;

    /// Access common fields shared between samplers
    fn common(&self) -> &Common;
    fn common_mut(&mut self) -> &mut Common;

    fn spawn(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>, handle: &Handle);

    /// Run the sampler and write new observations to the metrics library and
    /// wait until next sample interval
    async fn sample(&mut self) -> Result<(), std::io::Error>;

    /// Wait until the next time to sample
    fn delay(&mut self) -> &mut Option<Interval> {
        if self.common_mut().interval().is_none() {
            let duration = self
                .sampler_config()
                .interval()
                .unwrap_or_else(|| self.general_config().interval());
            self.common_mut()
                .set_interval(Some(interval(Duration::from_millis(duration as u64))));
        }
        self.common_mut().interval()
    }

    /// Access the specific sampler config
    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic>;

    /// Access the general config
    fn general_config(&self) -> &GeneralConfig {
        self.common().config().general()
    }

    /// Register all the statistics
    fn register(&self) {
        for statistic in self.sampler_config().statistics() {
            self.common()
                .metrics()
                .register(statistic, self.summary(statistic));
            self.common()
                .metrics()
                .register_output(statistic, Output::Reading);
            for percentile in self.sampler_config().percentiles() {
                self.common()
                    .metrics()
                    .register_output(statistic, Output::Percentile(*percentile));
            }
        }
    }

    fn summary(&self, _statistic: &Self::Statistic) -> Option<Summary> {
        None
    }

    fn metrics(&self) -> &Metrics<AtomicU32> {
        self.common().metrics()
    }
}

pub struct Common {
    config: Arc<Config>,
    interval: Option<Interval>,
    metrics: Arc<Metrics<AtomicU32>>,
}

impl Common {
    pub fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Self {
        Self {
            config,
            interval: None,
            metrics,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn interval(&mut self) -> &mut Option<Interval> {
        &mut self.interval
    }

    pub fn set_interval(&mut self, interval: Option<Interval>) {
        self.interval = interval
    }

    pub fn metrics(&self) -> &Metrics<AtomicU32> {
        &self.metrics
    }
}
