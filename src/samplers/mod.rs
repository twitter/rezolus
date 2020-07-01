// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use rustcommon_metrics::*;
use tokio::runtime::Handle;
use tokio::time::{interval, Interval};

use crate::config::General as GeneralConfig;
use crate::config::{Config, SamplerConfig};

pub mod cpu;
pub mod disk;
pub mod ext4;
pub mod http;
pub mod interrupt;
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
pub use http::Http;
pub use interrupt::Interrupt;
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
    fn new(common: Common) -> Result<Self, failure::Error>;

    /// Access common fields shared between samplers
    fn common(&self) -> &Common;
    fn common_mut(&mut self) -> &mut Common;

    fn spawn(common: Common);

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

    fn enabled(&self) -> bool {
        self.sampler_config().enabled()
    }

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
                .delete_output(statistic, Output::Reading);
            for percentile in self.sampler_config().percentiles() {
                self.common()
                    .metrics()
                    .delete_output(statistic, Output::Percentile(*percentile));
            }
        }
    }

    fn summary(&self, _statistic: &Self::Statistic) -> Option<Summary> {
        None
    }

    fn metrics(&self) -> &Metrics<AtomicU32> {
        self.common().metrics()
    }

    /// Used to map errors according to fault tolerance
    /// WouldBlock is returned as-is so that async/await behaves as expected
    /// All other errors are handled per fault tolerance setting
    fn map_result(&self, result: Result<(), std::io::Error>) -> Result<(), std::io::Error> {
        if let Err(e) = result {
            if e.kind() == std::io::ErrorKind::WouldBlock {
                return Err(e);
            }
            if self.common().config().general().fault_tolerant() {
                debug!("error: {}", e);
            } else {
                fatal!("error: {}", e);
            }
        }
        Ok(())
    }
}

pub struct Common {
    config: Arc<Config>,
    handle: Handle,
    interval: Option<Interval>,
    metrics: Arc<Metrics<AtomicU32>>,
}

impl Clone for Common {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            handle: self.handle.clone(),
            interval: None,
            metrics: self.metrics.clone(),
        }
    }
}

impl Common {
    pub fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>, handle: Handle) -> Self {
        Self {
            config,
            handle,
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
