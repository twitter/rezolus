#![allow(dead_code)]

use std::any::Any;
use std::borrow::Cow;
use std::ops::Deref;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crossbeam::atomic::AtomicCell;
use rustcommon_atomics::{Atomic, AtomicU32, AtomicU64};
use rustcommon_heatmap::AtomicHeatmap;
use rustcommon_metrics_v2::{Counter, DynBoxedMetric, Gauge, Metric};

use super::LazyMetric;

type Heatmap = AtomicHeatmap<u64, AtomicU32>;

pub struct SampledHeatmap {
    refreshed: AtomicCell<Option<Instant>>,
    reading: AtomicU64,
    heatmap: Heatmap,
    percentiles: Cow<'static, [f64]>,
}

impl SampledHeatmap {
    pub fn new(heatmap: Heatmap, percentiles: impl Into<Cow<'static, [f64]>>) -> Self {
        Self {
            refreshed: AtomicCell::new(None),
            reading: AtomicU64::new(0),
            heatmap,
            percentiles: percentiles.into(),
        }
    }

    pub fn percentiles(&self) -> &[f64] {
        &self.percentiles
    }

    pub fn heatmap(&self) -> &Heatmap {
        &self.heatmap
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
        self.heatmap.increment(time, rate as _, 1);
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
        self.heatmap.increment(time, rate as _, 1);
    }
}

impl Deref for SampledHeatmap {
    type Target = Heatmap;

    fn deref(&self) -> &Self::Target {
        &self.heatmap
    }
}

impl Metric for SampledHeatmap {
    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

/// A combination of two metrics: a counter and a heatmap of its rate of change
pub struct HeatmapSummarizedCounter {
    counter: DynBoxedMetric<LazyMetric<Counter>>,
    heatmap: DynBoxedMetric<LazyMetric<SampledHeatmap>>,
}

impl HeatmapSummarizedCounter {
    pub fn new(span: Duration, percentiles: &[f64]) -> Self {
        Self {
            counter: DynBoxedMetric::unregistered(LazyMetric::new(Counter::new())),
            heatmap: DynBoxedMetric::unregistered(LazyMetric::new(SampledHeatmap::new(
                AtomicHeatmap::new(1_000_000_000, 2, span, Duration::from_secs(1)),
                percentiles.to_owned(),
            ))),
        }
    }

    pub fn register(&mut self, name: &str) {
        self.counter.register(name.to_owned());
        self.heatmap.register(format!("{}/histogram", name));
    }

    pub fn counter(&self) -> &Counter {
        &self.counter
    }

    pub fn heatmap(&self) -> &SampledHeatmap {
        &self.heatmap
    }

    pub fn increment(&self, time: Instant) {
        self.add(time, 1)
    }

    pub fn add(&self, time: Instant, value: u64) {
        let value = self.counter.add(value) + value;
        self.heatmap.record_counter(time, value as _)
    }

    pub fn store(&self, time: Instant, value: u64) {
        self.counter.set(value);
        self.heatmap.record_counter(time, value);
    }
}

pub struct HeatmapSummarizedGauge {
    gauge: DynBoxedMetric<LazyMetric<Gauge>>,
    heatmap: DynBoxedMetric<LazyMetric<SampledHeatmap>>,
}

impl HeatmapSummarizedGauge {
    pub fn new(span: Duration, percentiles: &[f64]) -> Self {
        Self {
            gauge: DynBoxedMetric::unregistered(LazyMetric::new(Gauge::new())),
            heatmap: DynBoxedMetric::unregistered(LazyMetric::new(SampledHeatmap::new(
                AtomicHeatmap::new(1_000_000_000, 2, span, Duration::from_secs(1)),
                percentiles.to_owned(),
            ))),
        }
    }

    pub fn register(&mut self, name: &str) {
        self.gauge.register(name.to_owned());
        self.heatmap.register(format!("{}/histogram", name));
    }

    pub fn gauge(&self) -> &Gauge {
        &self.gauge
    }

    pub fn heatmap(&self) -> &SampledHeatmap {
        &self.heatmap
    }

    pub fn increment(&self, time: Instant) {
        self.add(time, 1)
    }

    pub fn add(&self, time: Instant, value: i64) {
        let value = self.gauge.add(value) + value;
        self.heatmap.record_gauge(time, value)
    }

    pub fn store(&self, time: Instant, value: i64) {
        self.gauge.set(value);
        self.heatmap.record_gauge(time, value);
    }
}
