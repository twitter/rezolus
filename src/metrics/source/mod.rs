// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

/// Defines the source for a given statistic
#[derive(PartialEq, Eq, Debug, Hash, Copy, Clone)]
pub enum Source {
    /// Indicates that the source is a monotonically incrementing count.
    Counter,
    /// Indicates that the source is an instantaneous gauge reading.
    Gauge,
    /// Indicates that the source is an underlying distribution (histogram).
    Distribution,
}
