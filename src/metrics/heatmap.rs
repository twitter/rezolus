// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(dead_code)]

use std::any::Any;
use std::borrow::Cow;
use std::ops::Deref;
use std::time::{Duration, Instant};

use rustcommon_atomics::AtomicU32;
use rustcommon_heatmap::AtomicHeatmap;
use rustcommon_metrics_v2::{DynBoxedMetric, Metric};

use super::LazyMetric;
use crate::samplers::CommonSamplerConfig;

type Heatmap = AtomicHeatmap<u64, AtomicU32>;

pub struct SampledHeatmap {
    heatmap: Heatmap,
    percentiles: Cow<'static, [f64]>,
}

impl SampledHeatmap {
    pub fn new(heatmap: Heatmap, percentiles: impl Into<Cow<'static, [f64]>>) -> Self {
        Self {
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

    pub fn increment(&self, time: Instant, value: u64, count: u32) {
        self.heatmap.increment(time, value, count)
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
pub struct SummarizedDistribution {
    heatmap: DynBoxedMetric<LazyMetric<SampledHeatmap>>,
}

impl SummarizedDistribution {
    pub fn with_config(config: &CommonSamplerConfig) -> Self {
        Self::new(config.span, &config.percentiles)
    }

    pub fn new(span: Duration, percentiles: &[f64]) -> Self {
        Self {
            heatmap: DynBoxedMetric::unregistered(LazyMetric::new(SampledHeatmap::new(
                Heatmap::new(1_000_000_000, 2, span, Duration::from_secs(1)),
                percentiles.to_owned(),
            ))),
        }
    }

    pub fn register(&mut self, name: &str) {
        self.heatmap.register(name.to_owned());
    }

    pub fn insert(&self, time: Instant, value: u64, count: u32) {
        self.heatmap.increment(time, value, count)
    }
}
