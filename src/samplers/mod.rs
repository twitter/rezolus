// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::any::type_name;
use std::convert::TryInto;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::runtime::Runtime;
use tokio::time::{interval, Interval};

use crate::config::General as GeneralConfig;
use crate::config::{Config, SamplerConfig};
use crate::*;

pub mod cpu;
pub mod disk;
pub mod ext4;
pub mod http;
pub mod interrupt;
pub mod krb5kdc;
pub mod memcache;
pub mod memory;
pub mod network;
pub mod ntp;
pub mod nvidia;
pub mod page_cache;
pub mod process;
pub mod rezolus;
pub mod scheduler;
pub mod softnet;
pub mod tcp;
pub mod udp;
pub mod usercall;
pub mod xfs;

pub use cpu::Cpu;
pub use disk::Disk;
pub use ext4::Ext4;
pub use http::Http;
pub use interrupt::Interrupt;
pub use krb5kdc::Krb5kdc;
pub use memcache::Memcache;
pub use memory::Memory;
pub use network::Network;
pub use ntp::Ntp;
pub use nvidia::Nvidia;
pub use page_cache::PageCache;
pub use process::Process;
pub use rezolus::Rezolus;
pub use scheduler::Scheduler;
pub use softnet::Softnet;
pub use tcp::Tcp;
pub use udp::Udp;
pub use usercall::Usercall;
pub use xfs::Xfs;

#[async_trait]
pub trait Sampler: Sized + Send {
    type Statistic: Statistic;

    /// Create a new instance of the sampler
    fn new(common: Common) -> Result<Self, anyhow::Error>;

    /// Access common fields shared between samplers
    fn common(&self) -> &Common;
    fn common_mut(&mut self) -> &mut Common;

    /// Run the sampler and write new observations to the metrics library and
    /// wait until next sample interval
    async fn sample(&mut self) -> Result<(), std::io::Error>;

    fn interval(&self) -> usize {
        self.sampler_config()
            .interval()
            .unwrap_or_else(|| self.general_config().interval())
    }

    /// Wait until the next time to sample
    fn delay(&mut self) -> &mut Option<Interval> {
        if self.common_mut().interval().is_none() {
            let millis = self.interval() as u64;
            self.common_mut()
                .set_interval(Some(interval(std::time::Duration::from_millis(millis))));
        }
        self.common_mut().interval()
    }

    /// Access the specific sampler config
    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        Self::config(self.common())
    }

    fn config(common: &Common) -> &dyn SamplerConfig<Statistic = Self::Statistic>;

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
                .add_output(&statistic, Output::Reading);
            let percentiles = self.sampler_config().percentiles();
            if !percentiles.is_empty() {
                if statistic.source() == Source::Distribution {
                    self.common().metrics().add_summary(
                        &statistic,
                        Summary::heatmap(
                            1_000_000_000,
                            2,
                            Duration::from_secs(
                                self.common()
                                    .config()
                                    .general()
                                    .window()
                                    .try_into()
                                    .unwrap(),
                            ),
                            Duration::from_secs(1),
                        ),
                    );
                } else {
                    self.common()
                        .metrics()
                        .add_summary(&statistic, Summary::stream(self.samples()));
                }
            }
            for percentile in percentiles {
                self.common()
                    .metrics()
                    .add_output(&statistic, Output::Percentile(*percentile));
            }
        }
    }

    fn samples(&self) -> usize {
        ((1000.0 / self.interval() as f64) * self.general_config().window() as f64).ceil() as usize
    }

    fn metrics(&self) -> &Metrics {
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

#[async_trait]
pub trait Spawner<T: Sampler> {
    fn spawn(common: Common);
}

impl<T: Sampler + 'static> Spawner<T> for T {
    fn spawn(common: Common) {
        if T::config(&common).enabled() {
            match T::new(common.clone()) {
                Ok(mut sampler) => {
                    common.runtime().spawn(async move {
                        loop {
                            let _ = sampler.sample().await;
                        }
                    });
                }
                Err(error) => {
                    let sampler_name = type_name::<T>().split(':').last().unwrap_or_default();
                    if !common.config.fault_tolerant() {
                        fatal!("failed to initialize sampler {sampler_name}: {error}");
                    } else {
                        error!("failed to initialize sampler {sampler_name}: {error}");
                    }
                }
            }
        }
    }
}

pub struct Common {
    config: Arc<Config>,
    runtime: Arc<Runtime>,
    hardware_info: Arc<HardwareInfo>,
    interval: Option<Interval>,
    metrics: Arc<Metrics>,
}

impl Clone for Common {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            runtime: self.runtime.clone(),
            hardware_info: self.hardware_info.clone(),
            interval: None,
            metrics: self.metrics.clone(),
        }
    }
}

impl Common {
    pub fn new(config: Arc<Config>, metrics: Arc<Metrics>, runtime: Arc<Runtime>) -> Self {
        Self {
            config,
            hardware_info: Arc::new(HardwareInfo::new()),
            interval: None,
            metrics,
            runtime,
        }
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn hardware_info(&self) -> &HardwareInfo {
        &self.hardware_info
    }

    pub fn interval(&mut self) -> &mut Option<Interval> {
        &mut self.interval
    }

    pub fn set_interval(&mut self, interval: Option<Interval>) {
        self.interval = interval
    }

    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}
