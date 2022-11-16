use std::io::Cursor;

use anyhow::Context;
use rustcommon_metrics::Counter;

use crate::{Encoder, Message};

#[metric(
    name = "encoder/zstd/txbytes_uncompressed",
    description = "the number of bytes sent to the kafka channel before being zstd compressed"
)]
static TXBYTES_UNCOMPRESSED: Counter = Counter::new();

/// An encoder wrapper which zstd-compresses all messages from the inner
/// encoder.
#[derive(Default)]
pub struct ZstdEncoder<E>(E);

impl<E: Encoder> ZstdEncoder<E> {
    pub fn new(encoder: E) -> Self {
        Self(encoder)
    }
}

impl<E: Encoder> Encoder for ZstdEncoder<E> {
    fn encode(&mut self, sample: augur_common::Sample) -> anyhow::Result<Option<Message>> {
        Ok(match self.0.encode(sample)? {
            Some(message) => {
                TXBYTES_UNCOMPRESSED.add(message.data.len() as _);
                let compressed = zstd::encode_all(Cursor::new(&message.data), 0)
                    .context("Failed to zstd-compress encoded message")?;

                Some(Message {
                    data: compressed,
                    timestamp: message.timestamp,
                })
            }
            None => None,
        })
    }
}
