use async_trait::async_trait;

use crate::Sample;

/// Trait for types which collect samples from the current host.
///
/// This covers any method by which samples can be collected from the underlying
/// system. That may be runing a profiler, using the perf-events API, or
/// something else entirely.
#[async_trait]
pub trait Collector: Send {
    /// Gather the next sample from the system.
    ///
    /// This should always block until the next sample is ready, cancellation
    /// will be handled at a higher level.
    async fn next_sample(&mut self) -> anyhow::Result<Sample>;
}

/// Sink for samples to be sent on to somewhere beyond the current host.
#[async_trait]
pub trait Emitter: Send + Sync {
    /// Emit the next sample.
    async fn emit_sample(&self, sample: Sample) -> anyhow::Result<()>;
}

/// Trait for types which add additional information to a sample.
///
/// This could include unwinding the sample stack, annotating it with the
/// systemd unit that it is running under, or anything else that adds
/// additional information to a sample.
#[async_trait]
pub trait Annotator: Send + Sync {
    fn name(&self) -> &str;

    /// Annotate a sample with more information.
    async fn annotate(&self, sample: &mut Sample);
}
