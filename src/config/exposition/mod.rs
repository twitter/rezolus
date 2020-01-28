// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod kafka;

use serde_derive::*;

use self::kafka::*;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exposition {
    #[serde(default)]
    kafka: Kafka,
}

impl Exposition {
    #[cfg(feature = "push_kafka")]
    pub fn kafka(&self) -> &Kafka {
        &self.kafka
    }
}
