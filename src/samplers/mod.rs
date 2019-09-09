// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub(crate) mod container;
pub(crate) mod cpu;
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
pub use self::disk::Disk;
pub use self::memcache::Memcache;
pub use self::network::Network;
#[cfg(feature = "perf")]
pub use self::perf::Perf;
pub use self::rezolus::Rezolus;
pub use self::softnet::Softnet;

use crate::config::Config;

use failure::Error;
use metrics::{AtomicU32, Histogram, Measurement, Output, Percentile, Recorder, Source};

/// `Sampler`s are used to get samples of a particular subsystem or component
/// The `Sampler` will perform the necessary actions to update the telemetry and
/// record updated values into the metrics `Recorder`
pub trait Sampler<'a> {
    fn new(
        config: &'a Config,
        recorder: &'a Recorder<AtomicU32>,
    ) -> Result<Option<Box<Self>>, Error>
    where
        Self: Sized;

    /// Perform required sampling steps and send stats to the `Recorder`
    fn sample(&mut self) -> Result<(), ()>;

    /// Return the name of the `Sampler`
    fn name(&self) -> String;

    /// Register any metrics that the `Sampler` will report
    fn register(&mut self);

    /// De-register any metrics for the `Sampler`
    fn deregister(&mut self);
}

pub trait Statistic: ToString + Sized {}


pub struct Common<'a> {
    config: &'a Config,
    recorder: &'a Recorder<AtomicU32>,
}

impl<'a> Common<'a> {
    pub fn new(config: &'a Config, recorder: &'a Recorder<AtomicU32>) -> Self {
        Self {
            config,
            recorder,
        }
    }

    pub fn config(&self) -> &'a Config {
        self.config
    }

    pub fn recorder(&self) -> &'a Recorder<AtomicU32> {
        self.recorder
    }

    pub fn delete_channel(&self, name: String) {
        self.recorder.delete_channel(name)
    }

    pub fn record_counter(&self, label: &dyn ToString, time: u64, value: u64) {
        self.recorder.record(label.to_string(), Measurement::Counter { time, value });
    }

    pub fn record_gauge(&self, label: &dyn ToString, time: u64, value: u64) {
        self.recorder.record(label.to_string(), Measurement::Gauge { time, value });
    }

    pub fn register_counter(
        &self,
        label: &dyn ToString,
        max: u64,
        precision: u32,
        percentiles: &[Percentile],
    ) {
        self.recorder.add_channel(
            label.to_string(),
            Source::Counter,
            Some(Histogram::new(max, precision, Some(self.config.general().window()), None)),
        );
        self.recorder.add_output(label.to_string(), Output::Counter);
        self.recorder.add_output(label.to_string(), Output::MaxPointTime);
        for percentile in percentiles {
            self.recorder.add_output(label.to_string(), Output::Percentile(*percentile));
        }
    }

    pub fn register_gauge(
        &self,
        label: &dyn ToString,
        max: u64,
        precision: u32,
        percentiles: &[Percentile],
    ) {
        self.recorder.add_channel(
            label.to_string(),
            Source::Gauge,
            Some(Histogram::new(max, precision, Some(self.config.general().window()), None)),
        );
        self.recorder.add_output(label.to_string(), Output::Counter);
        self.recorder.add_output(label.to_string(), Output::MaxPointTime);
        for percentile in percentiles {
            self.recorder.add_output(label.to_string(), Output::Percentile(*percentile));
        }
    }
}
