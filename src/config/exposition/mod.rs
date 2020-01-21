use serde_derive::*;

mod kafka;

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
