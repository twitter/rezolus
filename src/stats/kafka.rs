// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use kafka::producer::{Producer, Record};

use crate::config::Config;
use crate::stats::MetricsSnapshot;

use std::convert::TryInto;
use std::sync::Arc;
use std::time::{Duration, Instant};

use metrics::*;

pub struct KafkaProducer {
    snapshot: MetricsSnapshot,
    producer: Producer,
    topic: String,
    interval: Duration,
}

impl KafkaProducer {
    pub fn new(
        config: Arc<Config>,
        metrics: Metrics<AtomicU32>,
        count_label: Option<&str>,
    ) -> Self {
        Self {
            snapshot: MetricsSnapshot::new(metrics, count_label),
            producer: Producer::from_hosts(config.kafka().hosts())
                .create()
                .unwrap(),
            topic: config.kafka().topic().unwrap(),
            interval: Duration::from_millis(config.kafka().interval().try_into().unwrap()),
        }
    }

    pub fn run(&mut self) {
        let start = Instant::now();
        self.snapshot.refresh();
        self.producer
            .send(&Record::from_value(&self.topic, self.snapshot.json(false)));
        let stop = Instant::now();
        if start + self.interval > stop {
            std::thread::sleep(self.interval - (stop - start));
        }
    }
}
