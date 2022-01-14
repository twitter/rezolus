// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub trait FloatConvert {
    fn to_float(self) -> f64;
    fn from_float(value: f64) -> Self;
}

impl FloatConvert for u64 {
    fn to_float(self) -> f64 {
        self as f64
    }
    fn from_float(value: f64) -> Self {
        value as Self
    }
}

impl FloatConvert for u32 {
    fn to_float(self) -> f64 {
        self as f64
    }
    fn from_float(value: f64) -> Self {
        value as Self
    }
}

impl FloatConvert for u16 {
    fn to_float(self) -> f64 {
        self as f64
    }
    fn from_float(value: f64) -> Self {
        value as Self
    }
}

impl FloatConvert for u8 {
    fn to_float(self) -> f64 {
        self as f64
    }
    fn from_float(value: f64) -> Self {
        value as Self
    }
}
