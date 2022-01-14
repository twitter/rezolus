// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::metrics::error::SummaryError;
use crate::metrics::*;
use core::marker::PhantomData;

use rustcommon_atomics::Atomic;
use rustcommon_heatmap::{AtomicHeatmap, Duration, Instant};
use rustcommon_streamstats::AtomicStreamstats;

pub(crate) enum SummaryStruct<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    Heatmap(AtomicHeatmap<<Value as Atomic>::Primitive, Count>),
    Stream(AtomicStreamstats<Value>),
}

impl<Value, Count> SummaryStruct<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    pub fn increment(
        &self,
        time: Instant<Nanoseconds<u64>>,
        value: <Value as Atomic>::Primitive,
        count: <Count as Atomic>::Primitive,
    ) {
        match self {
            Self::Heatmap(heatmap) => heatmap.increment(time, value, count),
            Self::Stream(stream) => stream.insert(value),
        }
    }

    pub fn percentile(
        &self,
        percentile: f64,
    ) -> Result<<Value as Atomic>::Primitive, SummaryError> {
        match self {
            Self::Heatmap(heatmap) => heatmap.percentile(percentile).map_err(SummaryError::from),
            Self::Stream(stream) => stream.percentile(percentile).map_err(SummaryError::from),
        }
    }

    pub fn heatmap(
        max: <Value as Atomic>::Primitive,
        precision: u8,
        span: Duration<Nanoseconds<u64>>,
        resolution: Duration<Nanoseconds<u64>>,
    ) -> Self {
        Self::Heatmap(AtomicHeatmap::new(max, precision, span, resolution))
    }

    pub fn stream(samples: usize) -> Self {
        Self::Stream(AtomicStreamstats::new(samples))
    }
}

enum SummaryType<Value>
where
    Value: crate::Value,
    <Value as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive>,
{
    Heatmap(
        <Value as Atomic>::Primitive,
        u8,
        Duration<Nanoseconds<u64>>,
        Duration<Nanoseconds<u64>>,
    ),
    Stream(usize),
}

pub struct Summary<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    inner: SummaryType<Value>,
    _count: PhantomData<Count>,
}

impl<Value, Count> Summary<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    pub fn heatmap(
        max: <Value as Atomic>::Primitive,
        precision: u8,
        span: Duration<Nanoseconds<u64>>,
        resolution: Duration<Nanoseconds<u64>>,
    ) -> Summary<Value, Count> {
        Self {
            inner: SummaryType::Heatmap(max, precision, span, resolution),
            _count: PhantomData,
        }
    }

    pub fn stream(samples: usize) -> Summary<Value, Count> {
        Self {
            inner: SummaryType::Stream(samples),
            _count: PhantomData,
        }
    }

    pub(crate) fn build(&self) -> SummaryStruct<Value, Count> {
        match self.inner {
            SummaryType::Heatmap(max, precision, span, resolution) => {
                SummaryStruct::heatmap(max, precision, span, resolution)
            }
            SummaryType::Stream(samples) => SummaryStruct::stream(samples),
        }
    }
}
