// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(dead_code)]

mod channel;
mod entry;
mod error;
#[allow(clippy::module_inception)]
mod metrics;
mod outputs;
mod source;
mod summary;
mod traits;

pub use error::MetricsError;
pub use metrics::{Metric, Metrics};
pub use outputs::Output;
pub use source::Source;
pub use summary::Summary;
pub use traits::{Count, Primitive, Statistic, Value};

// Re-export atomic trait and types for convenience
pub use rustcommon_atomics::{Atomic, AtomicU16, AtomicU32, AtomicU64, AtomicU8};
// Re-export time types for convenience
pub use rustcommon_time::*;

#[cfg(test)]
mod tests {
    use super::*;

    enum TestStat {
        Alpha,
    }

    impl Statistic for TestStat {
        fn name(&self) -> &str {
            match self {
                Self::Alpha => "alpha",
            }
        }

        fn source(&self) -> Source {
            match self {
                Self::Alpha => Source::Counter,
            }
        }

        fn summary(&self) -> Option<Summary> {
            match self {
                Self::Alpha => Some(Summary::stream(1000)),
            }
        }
    }

    #[test]
    fn basic() {
        let metrics = Metrics::new();
        metrics.register(&TestStat::Alpha);
        assert!(metrics.reading(&TestStat::Alpha).is_err());
        metrics
            .record_counter(&TestStat::Alpha, Instant::<Nanoseconds<u64>>::now(), 0)
            .expect("failed to record counter");
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(0));
        let now = Instant::<Nanoseconds<u64>>::now();
        metrics
            .record_counter(&TestStat::Alpha, now + Duration::from_millis(500), 0)
            .expect("failed to record counter");
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(0));
        assert_eq!(metrics.percentile(&TestStat::Alpha, 0.0), Ok(0));
        metrics
            .record_counter(&TestStat::Alpha, now + Duration::from_millis(1500), 1)
            .expect("failed to record counter");
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(1));
        assert_eq!(metrics.percentile(&TestStat::Alpha, 100.0), Ok(1));
    }

    #[test]
    fn outputs() {
        let metrics = Metrics::new();
        metrics.register(&TestStat::Alpha);
        assert!(metrics.snapshot().is_empty());
        metrics.add_output(&TestStat::Alpha, Output::Reading);
        let _ = metrics.record_counter(&TestStat::Alpha, Instant::<Nanoseconds<u64>>::now(), 1);
        assert_eq!(metrics.snapshot().len(), 1);
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(1));
    }

    #[test]
    fn absolute_counter() {
        let metrics = Metrics::new();
        metrics.register(&TestStat::Alpha);
        let start = Instant::<Nanoseconds<u64>>::now();
        assert!(metrics.reading(&TestStat::Alpha).is_err());
        metrics.record_counter(&TestStat::Alpha, start, 0).unwrap();
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(0));
        metrics
            .record_counter(
                &TestStat::Alpha,
                start + Duration::from_millis(1000),
                1000000,
            )
            .unwrap();
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(1000000));
        assert_eq!(metrics.percentile(&TestStat::Alpha, 99.9), Ok(1000000));
        metrics
            .record_counter(
                &TestStat::Alpha,
                start + Duration::from_millis(2000),
                3000000,
            )
            .unwrap();
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(3000000));
        assert_eq!(metrics.percentile(&TestStat::Alpha, 99.9), Ok(2000000));
        metrics.record_counter(&TestStat::Alpha, start, 42).unwrap();
        assert_ne!(metrics.reading(&TestStat::Alpha), Ok(42));
    }

    #[test]
    fn increment_counter() {
        let metrics = Metrics::new();
        metrics.register(&TestStat::Alpha);
        assert!(metrics.reading(&TestStat::Alpha).is_err());
        metrics.increment_counter(&TestStat::Alpha, 1).unwrap();
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(1));
        metrics.increment_counter(&TestStat::Alpha, 0).unwrap();
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(1));
        metrics.increment_counter(&TestStat::Alpha, 10).unwrap();
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(11));
    }
}
