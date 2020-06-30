// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::convert::TryInto;
use std::sync::Arc;
use std::time::{Duration, Instant};

use kafka::producer::{Producer, Record};
use metrics::*;

use crate::config::Config;
use crate::exposition::MetricsSnapshot;

pub struct KafkaProducer {
    snapshot: MetricsSnapshot,
    producer: Producer,
    topic: String,
    interval: Duration,
}

impl KafkaProducer {
    pub fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Self {
        Self {
            snapshot: MetricsSnapshot::new(metrics, config.general().reading_suffix()),
            producer: Producer::from_hosts(config.exposition().kafka().hosts())
                .create()
                .unwrap(),
            topic: config.exposition().kafka().topic().unwrap(),
            interval: Duration::from_millis(
                config.exposition().kafka().interval().try_into().unwrap(),
            ),
        }
    }

    pub fn run(&mut self) {
        let start = Instant::now();
        self.snapshot.refresh();
        let _ = self
            .producer
            .send(&Record::from_value(&self.topic, self.snapshot.json(false)));
        let stop = Instant::now();
        if start + self.interval > stop {
            std::thread::sleep(self.interval - (stop - start));
        }
    }
}
