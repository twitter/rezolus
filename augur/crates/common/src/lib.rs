//! Common data structures used throughout augur.
//!
//! The important types from this crate are
//! - A [`Sample`] is what augur is trying to generate. Each one is a single
//!   profiling sample that will be emitted by augur.
//! - [`Collector`]s gather samples from the underlying system.
//! - [`Emitter`]s then send the fully annotated sample on to another remote
//!   system for aggregation and processing.
//! - [`Annotator`] is for types which gather extra data from the system or
//!   compute some further metadata based off previous annotations.
//!
//! The types in this crate are not meant to be static. If you are writing an
//! augur annotator that produces something that is not already part of
//! [`Sample`], [`Frame`], or any of the other types involved then it should
//! be added in where it makes sense to do so.

mod convert;
mod sample;
mod serde;
mod traits;

pub use self::sample::*;
pub use self::traits::*;
