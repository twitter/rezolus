// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

// Internal representation which approximates the percentile
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum ApproxOutput {
    Reading,
    Percentile(u64),
}

/// Defines an output that should be reported in a snapshot for a statistic
#[derive(Copy, Clone)]
pub enum Output {
    /// A counter or gauge reading
    Reading,
    /// A percentile from a statistic summary
    Percentile(f64),
}

impl From<Output> for ApproxOutput {
    fn from(output: Output) -> Self {
        match output {
            Output::Reading => Self::Reading,
            Output::Percentile(percentile) => {
                Self::Percentile((percentile * 1000000.0).ceil() as u64)
            }
        }
    }
}

impl From<ApproxOutput> for Output {
    fn from(output: ApproxOutput) -> Self {
        match output {
            ApproxOutput::Reading => Self::Reading,
            ApproxOutput::Percentile(percentile) => Self::Percentile(percentile as f64 / 1000000.0),
        }
    }
}
