// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! This crate provides a simple metrics library for Rust programs.
//!
//! # Example
//!
//! ```rust
//! use rustcommon_metrics::*;
//!
//! // Define a custom metric
//! enum MyMetric {
//!     Requests,
//!     Errors,
//! }
//!
//! impl Statistic for MyMetric {
//!     fn name(&self) -> &str {
//!         match self {
//!             Self::Requests => "requests",
//!             Self::Errors => "errors",
//!         }
//!     }
//!
//!     fn source(&self) -> Source {
//!         match self {
//!             Self::Requests => Source::Counter,
//!             Self::Errors => Source::Counter,
//!         }
//!     }
//!
//!     fn summary(&self) -> Option<Summary> {
//!         match self {
//!             Self::Requests => Some(Summary::stream(1000)),
//!             Self::Errors => None,
//!         }
//!     }
//! }
//!
//! fn main() {
//!     // Create a new metrics object
//!     let metrics = Metrics::new();
//!
//!     // Register the custom metric with the metrics object
//!     metrics.register(&MyMetric::Requests);
//!     metrics.register(&MyMetric::Errors);
//!
//!     // Record some data for the requests metric
//!     metrics.increment_counter(&MyMetric::Requests, 1).unwrap();
//!     metrics.increment_counter(&MyMetric::Requests, 1).unwrap();
//!     metrics.increment_counter(&MyMetric::Requests, 1).unwrap();
//!
//!     // Print out the current value of the requests metric
//!     println!("Requests: {}", metrics.reading(&MyMetric::Requests).unwrap());
//!
//!     // Record some data for the errors metric
//!     metrics.increment_counter(&MyMetric::Errors, 1).unwrap();
//!
//!     // Print out the current value of the errors metric
//!     println!("Errors: {}", metrics.reading(&MyMetric::Errors).unwrap());
//! }
//!
//! ```
//!
//! The example defines a custom metric called `MyMetric` and uses the metrics library to record and read data for that metric.

mod channel;
mod entry;
mod error;
mod metrics;
mod outputs;
mod source;
mod summary;
mod traits;

pub use
  error::MetricsError;
pub use
  metrics::
{
  Metric,
Metrics};
pub use
  outputs::Output;
pub use
  source::Source;
pub use
  summary::Summary;
pub use
  traits::
{
  Count,
    Primitive,
    Statistic,
Value};

// Re-export atomic trait and types for convenience
pub use
  rustcommon_atomics::
{
  Atomic,
    AtomicU16,
    AtomicU32,
    AtomicU64,
AtomicU8};
// Re-export time types for convenience
pub use
rustcommon_time::*;

#[cfg(test)]
mod
  tests
{
  use
  super::*;

  enum TestStat
  {
    Alpha,
  }

  impl
    Statistic for
    TestStat
  {
    fn
      name (&self)->&str
      {
       match self
       {
	Self::Alpha = >"alpha",
	}
       }

    fn source (&self)->Source
    {
     match self
     {
      Self::Alpha = >Source::Counter,
      }
     }

    fn summary (&self)->Option < Summary >
    {
     match self
     {
      Self::Alpha = >Some (Summary::stream (1000)),
      }
     }
  }

#[test]
  fn
    basic ()
  {
    // Initialize a new Metrics instance
    let metrics = Metrics::new ();

    // Register a TestStat with the metrics
    metrics.register (&TestStat::Alpha);

    // Verify that reading the TestStat at this point returns an error
    assert ! (metrics.reading (&TestStat::Alpha).is_err ());

    // Record a counter with value 0 at the current time for the TestStat
    metrics.record_counter (&TestStat::Alpha,
			    Instant::<Nanoseconds < u64 >>::now (),
			    0).expect ("failed to record counter");

    // Verify that the reading of the TestStat now returns 0
    assert_eq ! (metrics.reading (&TestStat::Alpha), Ok (0));

    // Record another counter with value 0 but at time half a second from now
    let
      now = Instant::<Nanoseconds < u64 >>::now ();
    metrics.record_counter (&TestStat::Alpha,
			    now + Duration::from_millis (500),
			    0).expect ("failed to record counter");

    // Verify that the reading of the TestStat still returns 0
    assert_eq ! (metrics.reading (&TestStat::Alpha), Ok (0));

    // Verify that the 0th percentile of the TestStat is 0
    assert_eq ! (metrics.percentile (&TestStat::Alpha, 0.0), Ok (0));

    // Record another counter with value 1 at time 1.5 seconds from now
    metrics.record_counter (&TestStat::Alpha,
			    now + Duration::from_millis (1500),
			    1).expect ("failed to record counter");

    // Verify that the reading of the TestStat now returns 1
    assert_eq ! (metrics.reading (&TestStat::Alpha), Ok (1));

    // Verify that the 100th percentile of the TestStat is 1
    assert_eq ! (metrics.percentile (&TestStat::Alpha, 100.0), Ok (1));
  }

#[test]
  fn
  outputs ()
  {
    // Initialize a new Metrics instance
    let
      metrics = Metrics::new ();

    // Register a TestStat with the metrics
    metrics.register (&TestStat::Alpha);

    // Verify that the snapshot is empty
    assert ! (metrics.snapshot ().is_empty ());

    // Add Output::Reading to TestStat's outputs
    metrics.add_output (&TestStat::Alpha, Output::Reading);

    // Record a counter with value 1 at the current time for the TestStat
    let
      _ =
      metrics.record_counter (&TestStat::Alpha,
			      Instant::<Nanoseconds < u64 >>::now (), 1);

    // Verify that the snapshot contains one entry
    assert_eq ! (metrics.snapshot ().len (), 1);

    // Verify that the reading of the TestStat returns 1
    assert_eq ! (metrics.reading (&TestStat::Alpha), Ok (1));
  }

#[test]
  fn
  absolute_counter ()
  {
    // Initialize a new Metrics instance
    let
      metrics = Metrics::new ();

    // Register a TestStat with the metrics
    metrics.register (&TestStat::Alpha);

    // Get the start time
    let
      start = Instant::<Nanoseconds < u64 >>::now ();

    // Verify that reading the TestStat at this point returns an error
    assert ! (metrics.reading (&TestStat::Alpha).is_err ());

    // Record a counter with value 0 at the start time for the TestStat
    metrics.record_counter (&TestStat::Alpha, start, 0).unwrap ();

    // Verify that the reading of the TestStat now returns 0
    assert_eq ! (metrics.reading (&TestStat::Alpha), Ok (1));

    // Test that incrementing a counter updates the reading
    metrics.increment_counter (&TestStat::Alpha, 2).unwrap ();
    assert_eq ! (metrics.reading (&TestStat::Alpha), Ok (3));

// Test that recording a value updates the reading and summary
    let
      now = Instant::<Nanoseconds < u64 >>::now ();
    metrics.record_value (&TestStat::Alpha,
			  now + Duration::from_millis (2000),
			  4.2).expect ("failed to record value");
    assert_eq ! (metrics.reading (&TestStat::Alpha), Ok (4));
    assert_eq ! (metrics.percentile (&TestStat::Alpha, 0.0), Ok (4));
    assert_eq ! (metrics.percentile (&TestStat::Alpha, 100.0), Ok (4));

// Test that removing an output does not affect readings or summary
    let mut
    output = Output::new (|_ | { });
    output.add_format (OutputFormat::Json);
    metrics.add_output (&TestStat::Alpha, output.clone ());
    let
      output_id = output.id ();
    assert_eq ! (metrics.snapshot ().len (), 1);
    metrics.remove_output (output_id);
    assert_eq ! (metrics.snapshot ().len (), 0);
    assert_eq ! (metrics.reading (&TestStat::Alpha), Ok (4));
    assert_eq ! (metrics.percentile (&TestStat::Alpha, 50.0), Ok (4));
