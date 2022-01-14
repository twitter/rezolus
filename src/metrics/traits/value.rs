// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_atomics::*;

/// Value types may be used to store the primary value for a metric. For example
/// counter readings, gauge readings, or buckets values from underlying
/// distributions. Lower precision atomics help reduce in-memory representation
/// for stored values and streaming summaries, but are unable to represent large
/// counter and gauge values.
pub trait Value: Atomic + Arithmetic + Default {}

impl Value for AtomicU8 {}
impl Value for AtomicU16 {}
impl Value for AtomicU32 {}
impl Value for AtomicU64 {}
