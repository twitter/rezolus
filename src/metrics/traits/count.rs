// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_atomics::*;
use rustcommon_heatmap::AtomicCounter;

/// Count types are used internally for some types of summary datastructures,
/// such as heatmaps. The selected atomic is used as the internal counter width.
/// A well matched type would be large enough to hold maximum number of
/// observations that would fall into the same bucket in a heatmap. Using types
/// that are oversized will result in higher memory utilization for heatmap
/// summaries, but has no effect on basic counter/gauge values or streaming
/// summary sizes.
pub trait Count: Atomic + Default + AtomicCounter {}

impl Count for AtomicU8 {}
impl Count for AtomicU16 {}
impl Count for AtomicU32 {}
impl Count for AtomicU64 {}
