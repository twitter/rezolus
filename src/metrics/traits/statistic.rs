// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::metrics::{Primitive, Source, Summary};
use rustcommon_atomics::Atomic;

use core::hash::{Hash, Hasher};

/// A statistic represents a named entity that has associated measurements which
/// are recorded and metrics which are reported. This trait defines a set of
/// methods which uniquely identify the statistic, help the metrics library
/// track it appropriately, and allow including metadata in the exposition
/// format.
pub trait Statistic<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    /// The name is used to lookup the channel for the statistic and should be
    /// unique for each statistic. This field is used to hash the statistic in
    /// the core structure.
    fn name(&self) -> &str;
    /// Indicates which source type the statistic tracks.
    fn source(&self) -> Source;
    /// Optionally, specify a summary builder which configures a summary
    /// aggregation for producing additional metrics such as percentiles.
    fn summary(&self) -> Option<Summary<Value, Count>> {
        None
    }
}

impl<Value, Count> Hash for dyn Statistic<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().to_string().hash(state);
    }
}

impl<Value, Count> PartialEq for dyn Statistic<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl<Value, Count> Eq for dyn Statistic<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
}
