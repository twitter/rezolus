// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::metrics::channel::Channel;
use crate::metrics::entry::Entry;
use crate::metrics::outputs::ApproxOutput;
use crate::metrics::*;

use core::hash::{Hash, Hasher};

use dashmap::DashMap;
use rustcommon_atomics::*;
use rustcommon_time::Instant;

use std::collections::HashMap;

/// `Metrics` serves as a registry of outputs which are included in snapshots.
/// In addition, it serves as the core storage of measurements and summary
/// producing aggregation structures. It is designed for concurrent access,
/// making it useful for serving as a unified metrics library in multi-threaded
/// applications.
pub struct Metrics<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    channels: DashMap<String, Channel<Value, Count>>,
}

impl<'a, Value: 'a, Count: 'a> Default for Metrics<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    fn default() -> Self {
        Self {
            channels: DashMap::new(),
        }
    }
}

impl<'a, Value: 'a, Count: 'a> Metrics<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    /// Create a new empty metrics registry
    pub fn new() -> Self {
        Default::default()
    }

    /// Begin tracking a new statistic without a corresponding output. Useful if
    /// metrics will be retrieved and reported manually in a command-line tool.
    pub fn register(&self, statistic: &'a (dyn Statistic<Value, Count> + 'a)) {
        if !self.channels.contains_key(statistic.name()) {
            let channel = Channel::new(statistic);
            self.channels.insert(statistic.name().to_string(), channel);
        }
    }

    /// Stop tracking a statistics and any corresponding outputs.
    pub fn deregister(&self, statistic: &'a (dyn Statistic<Value, Count> + 'a)) {
        self.channels.remove(statistic.name());
    }

    /// Adds a new output to the registry which will be included in future
    /// snapshots. If the statistic is not already tracked, it will be
    /// registered.
    pub fn add_output(&self, statistic: &'a (dyn Statistic<Value, Count> + 'a), output: Output) {
        self.register(statistic);
        if let Some(channel) = self.channels.get_mut(statistic.name()) {
            channel.add_output(output);
        }
    }

    /// Remove an output from the registry so that it will not be included in
    /// future snapshots. This will not remove the related datastructures for
    /// the statistic even if no outputs remain. Use `deregister` method to stop
    /// tracking a statistic entirely.
    pub fn remove_output(&self, statistic: &dyn Statistic<Value, Count>, output: Output) {
        if let Some(channel) = self.channels.get_mut(statistic.name()) {
            channel.remove_output(output);
        }
    }

    /// Set the `Summary` for an already registered `Statistic`. This can be
    /// used when the parameters are not known at compile time. For example, if
    /// a sampling rate is user configurable at runtime, the number of samples
    /// may need to be higher for stream summaries.
    pub fn set_summary(
        &self,
        statistic: &'a (dyn Statistic<Value, Count> + 'a),
        summary: Summary<Value, Count>,
    ) {
        if let Some(mut channel) = self.channels.get_mut(statistic.name()) {
            channel.set_summary(summary);
        }
    }

    /// Conditionally add a `Summary` for a `Statistic` if one is not currently
    /// set. This may be used for dynamically registered statistic types to
    /// prevent clearing an existing summary.
    pub fn add_summary(
        &self,
        statistic: &'a (dyn Statistic<Value, Count> + 'a),
        summary: Summary<Value, Count>,
    ) {
        if let Some(mut channel) = self.channels.get_mut(statistic.name()) {
            channel.add_summary(summary);
        }
    }

    /// Remove all statistics and outputs.
    pub fn clear(&self) {
        self.channels.clear();
    }

    /// Record a bucket value + count pair for distribution based statistics.
    /// Use this when the data points are taken from a histogram and the summary
    /// for the statistic is a heatmap.
    pub fn record_bucket(
        &self,
        statistic: &'a (dyn Statistic<Value, Count> + 'a),
        time: Instant<Nanoseconds<u64>>,
        value: <Value as Atomic>::Primitive,
        count: <Count as Atomic>::Primitive,
    ) -> Result<(), MetricsError> {
        if statistic.source() == Source::Distribution {
            if let Some(channel) = self.channels.get(statistic.name()) {
                channel.record_bucket(time, value, count)
            } else {
                // statistic not registered
                Err(MetricsError::NotRegistered)
            }
        } else {
            // source mismatch
            Err(MetricsError::SourceMismatch)
        }
    }

    /// Record a counter observation for counter based statistics. May be used
    /// with any summary type. Summaries will track secondly rates for counter
    /// changes.
    pub fn record_counter(
        &self,
        statistic: &'a (dyn Statistic<Value, Count> + 'a),
        time: Instant<Nanoseconds<u64>>,
        value: <Value as Atomic>::Primitive,
    ) -> Result<(), MetricsError> {
        if statistic.source() == Source::Counter {
            if let Some(channel) = self.channels.get(statistic.name()) {
                channel.record_counter(time, value);
                Ok(())
            } else {
                // statistic not registered
                Err(MetricsError::NotRegistered)
            }
        } else {
            // source mismatch
            Err(MetricsError::SourceMismatch)
        }
    }

    /// Increment a counter by some amount. Wraps around on overflow. Currently,
    /// no summary statistics are calculated for increments to avoid complexity
    /// with out-of-order increments.
    pub fn increment_counter(
        &self,
        statistic: &'a (dyn Statistic<Value, Count> + 'a),
        value: <Value as Atomic>::Primitive,
    ) -> Result<(), MetricsError> {
        if statistic.source() == Source::Counter {
            if let Some(channel) = self.channels.get(statistic.name()) {
                channel.increment_counter(value);
                Ok(())
            } else {
                // statistic not registered
                Err(MetricsError::NotRegistered)
            }
        } else {
            // source mismatch
            Err(MetricsError::SourceMismatch)
        }
    }

    /// Record a gauge observation for gauge based statistics. May be used with
    /// any summary type. Summary tracks instantaneous gauge readings.
    pub fn record_gauge(
        &self,
        statistic: &'a (dyn Statistic<Value, Count> + 'a),
        time: Instant<Nanoseconds<u64>>,
        value: <Value as Atomic>::Primitive,
    ) -> Result<(), MetricsError> {
        if statistic.source() == Source::Gauge {
            if let Some(channel) = self.channels.get(statistic.name()) {
                channel.record_gauge(time, value);
                Ok(())
            } else {
                // statistic not registered
                Err(MetricsError::NotRegistered)
            }
        } else {
            // source mismatch
            Err(MetricsError::SourceMismatch)
        }
    }

    /// Return a percentile for the given statistic. For counters, it is the
    /// percentile of secondly rates across the summary. For gauges, it is the
    /// percentile of gauge readings observed across the summary. For
    /// distributions it is the percentile across the configured summary.
    pub fn percentile(
        &self,
        statistic: &'a (dyn Statistic<Value, Count> + 'a),
        percentile: f64,
    ) -> Result<<Value as Atomic>::Primitive, MetricsError> {
        if let Some(channel) = self.channels.get(statistic.name()) {
            channel.percentile(percentile)
        } else {
            Err(MetricsError::NotRegistered)
        }
    }

    /// Return the reading for the statistic. For counters and gauges, this is
    /// the most recent measurement recorded.
    // TODO: decide on how to handle distribution channels
    pub fn reading(
        &self,
        statistic: &'a (dyn Statistic<Value, Count> + 'a),
    ) -> Result<<Value as Atomic>::Primitive, MetricsError> {
        if let Some(channel) = self.channels.get(statistic.name()) {
            channel.reading()
        } else {
            Err(MetricsError::NotRegistered)
        }
    }

    /// Generates a point-in-time snapshot of metric and value pairs.
    pub fn snapshot(&self) -> HashMap<Metric<Value, Count>, <Value as Atomic>::Primitive> {
        #[allow(unused_mut)]
        let mut result = HashMap::new();
        for entry in &self.channels {
            let (_name, channel) = entry.pair();
            for output in channel.outputs() {
                if let Ok(value) = match Output::from(output) {
                    Output::Reading => {
                        self.reading(channel.statistic() as &dyn Statistic<Value, Count>)
                    }
                    Output::Percentile(percentile) => {
                        self.percentile(channel.statistic(), percentile)
                    }
                } {
                    result.insert(
                        Metric {
                            statistic: Entry::from(channel.statistic()),
                            output,
                        },
                        value,
                    );
                }
            }
        }
        result
    }
}

/// A statistic and output pair which has a corresponding value
// #[derive(PartialEq, Eq, Hash)]
pub struct Metric<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    statistic: Entry<Value, Count>,
    output: ApproxOutput,
}

impl<Value, Count> Hash for Metric<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.statistic.name().hash(state);
        self.output.hash(state);
    }
}

impl<Value, Count> PartialEq for Metric<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    fn eq(&self, other: &Self) -> bool {
        self.statistic.name() == other.statistic.name() && self.output == other.output
    }
}

impl<Value, Count> Eq for Metric<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
}

impl<Value, Count> Metric<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    /// Get the statistic name for the metric
    pub fn statistic(&self) -> &dyn Statistic<Value, Count> {
        &self.statistic as &dyn Statistic<Value, Count>
    }

    /// Get the output
    pub fn output(&self) -> Output {
        Output::from(self.output)
    }
}
