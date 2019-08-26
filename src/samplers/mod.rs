// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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

pub use self::cpu::Cpu;
pub use self::disk::Disk;
pub use self::memcache::Memcache;
pub use self::network::Network;
#[cfg(feature = "perf")]
pub use self::perf::Perf;
pub use self::rezolus::Rezolus;
pub use self::softnet::Softnet;
use failure::Error;

use crate::config::Config;
use metrics::{AtomicU32, Recorder};

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
