// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::any::Any;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

mod heatmap;

pub use self::heatmap::{HeatmapSummarizedCounter, HeatmapSummarizedGauge, SampledHeatmap};
use rustcommon_metrics_v2::Metric;
pub use rustcommon_metrics_v2::{metric, Counter, DynBoxedMetric, Gauge, Heatmap};

/// A short form for a sequence of if statements.
///
/// # Example
/// ```
/// # let i = 0;
/// if_block! {
///     if i % 2 == 0 => println!("divisible by 2");
///     if i % 3 == 0 => println!("divisible by 3");
///     if i % 4 == 0 => println!("divisible by 4");
///     // etc..
/// }
/// ```
macro_rules! if_block {
    { if let $pat:pat = $val:expr => $then:expr ; $( $rest:tt )* } => {{
    if let $pat = $val { $then; }
    if_block! { $( $rest )* }
    }};
    { if $cond:expr => $then:expr ; $( $rest:tt )* } => {{
        if $cond { $then; }
        if_block! { $( $rest )* }
    }};
    {} => {};
}

/// A metric that isn't enabled until it is first accessed.
///
/// When used as a static metric it won't allow retrieving a `&dyn Any` until
/// it has been accessed at least once.
pub struct LazyMetric<M> {
    metric: M,
    active: AtomicBool,
}

impl<M> LazyMetric<M> {
    pub const fn new(metric: M) -> Self {
        Self {
            metric,
            active: AtomicBool::new(false),
        }
    }

    pub fn force(this: &Self) -> &M {
        this.active.store(true, Ordering::Relaxed);
        &this.metric
    }

    pub fn get(this: &Self) -> Option<&M> {
        match this.active.load(Ordering::Relaxed) {
            true => Some(&this.metric),
            false => None,
        }
    }
}

impl<M: Metric> Metric for LazyMetric<M> {
    fn is_enabled(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Self::get(self).map(|x| x as &dyn Any)
    }
}

impl<M> Deref for LazyMetric<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        Self::force(self)
    }
}

impl<M> DerefMut for LazyMetric<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        *self.active.get_mut() = true;
        &mut self.metric
    }
}

impl<M: Default> Default for LazyMetric<M> {
    fn default() -> Self {
        Self {
            metric: M::default(),
            active: AtomicBool::new(false),
        }
    }
}
