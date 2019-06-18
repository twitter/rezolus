// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::Config;
use metrics::Recorder;

/// `Sampler`s are used to get samples of a particular subsystem or component
/// The `Sampler` will send `Message`s across the `Sender` for aggregation by
/// the stats library, `tock`
pub trait Sampler {
    /// Perform required sampling steps and send stats to the `Recorder`
    fn sample(&mut self, recorder: &Recorder<u32>, config: &Config) -> Result<(), SamplerError>;

    /// Return the name of the `Sampler`
    fn name(&self) -> String;

    /// Register any metrics that the `Sampler` will report
    fn register(&mut self, recorder: &Recorder<u32>, config: &Config);

    /// De-register any metrics for the `Sampler`
    fn deregister(&mut self, recorder: &Recorder<u32>, config: &Config);
}

/// Categorize the errors that a `Sampler` may return
pub enum SamplerError {
    Fatal,
}
