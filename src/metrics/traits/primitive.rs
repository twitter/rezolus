// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::metrics::traits::*;

use rustcommon_heatmap::Indexing;

use core::ops::Sub;

/// A trait that is used to track primitive types that correspond to supported
/// atomic types.
pub trait Primitive:
    Ord + Indexing + Copy + From<u8> + Sub<Self, Output = Self> + FloatConvert
{
}

impl Primitive for u8 {}
impl Primitive for u16 {}
impl Primitive for u32 {}
impl Primitive for u64 {}
