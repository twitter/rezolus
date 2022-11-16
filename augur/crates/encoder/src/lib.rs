//!

use std::time::SystemTime;

use augur_common::Sample;

#[macro_use]
extern crate rustcommon_metrics;

mod thrift;
mod zstd;

pub use crate::thrift::ThriftEncoder;
pub use crate::zstd::ZstdEncoder;

/// An encoder which first encodes the message using thrift and then compresses
/// it using zstd.
pub type ZstdThriftEncoder = ZstdEncoder<ThriftEncoder>;

/// Representation of a message being sent out to kafka.
pub struct Message {
    pub data: Vec<u8>,
    pub timestamp: SystemTime,
}

/// Encoder for messages before they are sent out to kafka.
///
/// Encoders can optionally batch together multiple samples by returning none
/// for the initial samples and finally returning a single message that covers
/// all of batched messages. Note that there are limitations on the size of
/// messages that can be sent to kafka.
pub trait Encoder {
    /// Encode a sample to bytes so that it can be sent to kafka.
    fn encode(&mut self, sample: Sample) -> anyhow::Result<Option<Message>>;
}
