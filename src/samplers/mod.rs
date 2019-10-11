// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub(crate) mod container;
pub(crate) mod cpu;
pub(crate) mod cpuidle;
pub(crate) mod disk;
#[cfg(feature = "ebpf")]
pub(crate) mod ebpf;
pub(crate) mod memcache;
pub(crate) mod network;
#[cfg(feature = "perf")]
pub(crate) mod perf;
pub(crate) mod rezolus;
pub(crate) mod softnet;

pub use self::container::Container;
pub use self::cpu::Cpu;
pub use self::cpuidle::CpuIdle;
pub use self::disk::Disk;
pub use self::memcache::Memcache;
pub use self::network::Network;
#[cfg(feature = "perf")]
pub use self::perf::Perf;
pub use self::rezolus::Rezolus;
pub use self::softnet::Softnet;

use crate::config::Config;

use failure::Error;
use metrics::*;

/// `Sampler`s are used to get samples of a particular subsystem or component
/// The `Sampler` will perform the necessary actions to update the telemetry and
/// record updated values into the metrics `Recorder`
pub trait Sampler<'a> {
    fn new(config: &'a Config, metrics: &'a Metrics<AtomicU32>) -> Result<Option<Box<Self>>, Error>
    where
        Self: Sized;

    /// Return a reference to the `Common` struct
    fn common(&self) -> &Common<'a>;

    /// Return the name of the `Sampler`
    fn name(&self) -> String;

    /// Perform required sampling steps and send stats to the `Recorder`
    fn sample(&mut self) -> Result<(), ()>;

    /// Return the current configured interval in milliseconds for the `Sampler`
    fn interval(&self) -> usize;

    /// Register any metrics that the `Sampler` will report
    fn register(&mut self);

    /// De-register any metrics for the `Sampler`
    fn deregister(&mut self);
}

pub struct Common<'a> {
    config: &'a Config,
    initialized: AtomicBool,
    metrics: &'a Metrics<AtomicU32>,
}

impl<'a> Common<'a> {
    pub fn new(config: &'a Config, metrics: &'a Metrics<AtomicU32>) -> Self {
        Self {
            config,
            initialized: AtomicBool::new(false),
            metrics,
        }
    }

    pub fn config(&self) -> &'a Config {
        self.config
    }

    pub fn initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    pub fn set_initialized(&self, value: bool) {
        self.initialized.store(value, Ordering::SeqCst);
    }

    pub fn metrics(&self) -> &'a Metrics<AtomicU32> {
        self.metrics
    }

    pub fn delete_channel(&self, name: &dyn ToString) {
        self.metrics.delete_channel(name.to_string())
    }

    #[allow(dead_code)]
    pub fn record_distribution(&self, label: &dyn ToString, time: u64, value: u64, count: u32) {
        self.metrics.record(
            label.to_string(),
            Measurement::Distribution { time, value, count },
        );
    }

    pub fn record_counter(&self, label: &dyn ToString, time: u64, value: u64) {
        self.metrics
            .record(label.to_string(), Measurement::Counter { time, value });
    }

    pub fn record_gauge(&self, label: &dyn ToString, time: u64, value: u64) {
        self.metrics
            .record(label.to_string(), Measurement::Gauge { time, value });
    }

    #[allow(dead_code)]
    pub fn register_distribution(
        &self,
        label: &dyn ToString,
        max: u64,
        precision: u32,
        percentiles: &[Percentile],
    ) {
        self.metrics.add_channel(
            label.to_string(),
            Source::Distribution,
            Some(Histogram::new(
                max,
                precision,
                Some(self.config.general().window()),
                None,
            )),
        );
        self.metrics.add_output(label.to_string(), Output::Counter);
        self.metrics
            .add_output(label.to_string(), Output::MaxPointTime);
        for percentile in percentiles {
            self.metrics
                .add_output(label.to_string(), Output::Percentile(*percentile));
        }
    }

    pub fn register_counter(
        &self,
        label: &dyn ToString,
        max: u64,
        precision: u32,
        percentiles: &[Percentile],
    ) {
        self.metrics.add_channel(
            label.to_string(),
            Source::Counter,
            Some(Histogram::new(
                max,
                precision,
                Some(self.config.general().window()),
                None,
            )),
        );
        self.metrics.add_output(label.to_string(), Output::Counter);
        self.metrics
            .add_output(label.to_string(), Output::MaxPointTime);
        for percentile in percentiles {
            self.metrics
                .add_output(label.to_string(), Output::Percentile(*percentile));
        }
    }

    pub fn register_gauge(
        &self,
        label: &dyn ToString,
        max: u64,
        precision: u32,
        percentiles: &[Percentile],
    ) {
        self.metrics.add_channel(
            label.to_string(),
            Source::Gauge,
            Some(Histogram::new(
                max,
                precision,
                Some(self.config.general().window()),
                None,
            )),
        );
        self.metrics.add_output(label.to_string(), Output::Counter);
        self.metrics
            .add_output(label.to_string(), Output::MaxPointTime);
        for percentile in percentiles {
            self.metrics
                .add_output(label.to_string(), Output::Percentile(*percentile));
        }
    }
}
