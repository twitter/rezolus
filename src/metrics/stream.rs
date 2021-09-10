// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(dead_code)]

use std::any::Any;
use std::borrow::Cow;
use std::ops::Deref;
use std::sync::atomic::Ordering;
use std::time::Instant;

use crossbeam::atomic::AtomicCell;
use rustcommon_atomics::{Atomic, AtomicU32, AtomicU64};
use rustcommon_heatmap::AtomicHeatmap;
use rustcommon_metrics_v2::{Counter, DynBoxedMetric, Gauge, Metric};

use super::LazyMetric;
use rustcommon_streamstats::AtomicStreamstats;

type Heatmap = AtomicHeatmap<u64, AtomicU32>;

pub struct SampledStream {
    refreshed: AtomicCell<Option<Instant>>,
    reading: AtomicU64,
    stream: AtomicStreamstats<AtomicU64>,
    percentiles: Cow<'static, [f64]>,
}

impl SampledStream {
    pub fn new(
        stream: AtomicStreamstats<AtomicU64>,
        percentiles: impl Into<Cow<'static, [f64]>>,
    ) -> Self {
        Self {
            refreshed: AtomicCell::new(None),
            reading: AtomicU64::new(0),
            stream,
            percentiles: percentiles.into(),
        }
    }

    pub fn percentiles(&self) -> &[f64] {
        &self.percentiles
    }

    pub fn record_counter(&self, time: Instant, value: u64) {
        let t0 = match self.refreshed.load() {
            Some(t0) if time <= t0 => return,
            Some(t0) => t0,
            None => {
                self.refreshed.store(Some(time));
                self.reading.store(value, Ordering::Release);
                return;
            }
        };

        self.refreshed.store(Some(time));
        let v0 = self.reading.swap(value, Ordering::Release);
        let dt = time - t0;
        let dv = (value - v0) as f64;
        let rate = (dv / dt.as_secs_f64()).ceil();
        self.stream.insert(rate as _);
    }

    pub fn record_gauge(&self, time: Instant, value: i64) {
        let t0 = match self.refreshed.load() {
            Some(t0) if time <= t0 => return,
            Some(t0) => t0,
            None => {
                self.refreshed.store(Some(time));
                self.reading.store(value as _, Ordering::Release);
                return;
            }
        };

        self.refreshed.store(Some(time));
        let v0 = self.reading.swap(value as _, Ordering::Release) as i64;
        let dt = time - t0;
        let dv = (value - v0) as f64;
        let rate = (dv / dt.as_secs_f64()).ceil();
        self.stream.insert(rate as _);
    }
}

impl Deref for SampledStream {
    type Target = AtomicStreamstats<AtomicU64>;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl Metric for SampledStream {
    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

/// A combination of two metrics: a counter and a heatmap of its rate of change
pub struct StreamSummarizedCounter {
    counter: DynBoxedMetric<LazyMetric<Counter>>,
    stream: DynBoxedMetric<LazyMetric<SampledStream>>,
}

impl StreamSummarizedCounter {
    pub fn new(capacity: usize, percentiles: &[f64]) -> Self {
        Self {
            counter: DynBoxedMetric::unregistered(LazyMetric::new(Counter::new())),
            stream: DynBoxedMetric::unregistered(LazyMetric::new(SampledStream::new(
                AtomicStreamstats::new(capacity),
                percentiles.to_owned(),
            ))),
        }
    }

    pub fn register(&mut self, name: &str) {
        self.counter.register(name.to_owned());
        self.stream.register(name.to_owned());
    }

    pub fn counter(&self) -> &Counter {
        &self.counter
    }

    pub fn increment(&self, time: Instant) {
        self.add(time, 1)
    }

    pub fn add(&self, time: Instant, value: u64) {
        let value = self.counter.add(value) + value;
        self.stream.record_counter(time, value as _)
    }

    pub fn store(&self, time: Instant, value: u64) {
        self.counter.set(value);
        self.stream.record_counter(time, value);
    }
}

pub struct StreamSummarizedGauge {
    gauge: DynBoxedMetric<LazyMetric<Gauge>>,
    stream: DynBoxedMetric<LazyMetric<SampledStream>>,
}

impl StreamSummarizedGauge {
    pub fn new(capacity: usize, percentiles: &[f64]) -> Self {
        Self {
            gauge: DynBoxedMetric::unregistered(LazyMetric::new(Gauge::new())),
            stream: DynBoxedMetric::unregistered(LazyMetric::new(SampledStream::new(
                AtomicStreamstats::new(capacity),
                percentiles.to_owned(),
            ))),
        }
    }

    pub fn register(&mut self, name: &str) {
        self.gauge.register(name.to_owned());
        self.stream.register(name.to_owned());
    }

    pub fn gauge(&self) -> &Gauge {
        &self.gauge
    }

    pub fn increment(&self, time: Instant) {
        self.add(time, 1)
    }

    pub fn add(&self, time: Instant, value: i64) {
        let value = self.gauge.add(value) + value;
        self.stream.record_gauge(time, value)
    }

    pub fn store(&self, time: Instant, value: i64) {
        self.gauge.set(value);
        self.stream.record_gauge(time, value);
    }
}
