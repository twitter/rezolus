use std::io::Cursor;

use augur_thrift::StackSample;
use thrift::protocol::TBinaryOutputProtocol;

use crate::{Encoder, Message};

/// An encoder that encodes the sample as a thrift struct.
#[derive(Default)]
pub struct ThriftEncoder(());

impl Encoder for ThriftEncoder {
    fn encode(&mut self, sample: augur_common::Sample) -> anyhow::Result<Option<Message>> {
        let timestamp = sample.time;
        let sample = StackSample::from(sample);

        // This initial capacity is larger than the average sample size so we
        // should avoid most reallocations.
        let mut buffer = Vec::with_capacity(2048);
        let mut protocol = TBinaryOutputProtocol::new(Cursor::new(&mut buffer), true);
        sample.write_to_out_protocol(&mut protocol)?;

        Ok(Some(Message {
            timestamp,
            data: buffer,
        }))
    }
}
