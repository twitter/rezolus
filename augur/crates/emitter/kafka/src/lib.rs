//! This crate contains a sample emitter that writes out zstd-compressed
//! messages to a kafka topic.
//!
//! It discovers the brokers from zookeeper. It will also emit all samples
//! to a single partition within the topic. In addition, it refreshes the list
//! of partitions (roughly) every hour so that if the topic is resized
//! the new partitions will be picked up automatically.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, Context};
use augur_common::{Emitter, Sample};
use augur_encoder::Encoder;
use rand::Rng;
use rdkafka::config::FromClientConfigAndContext;
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use rdkafka::util::Timeout;
use rustcommon_metrics::{metric, Counter};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::metrics::AugurProducerContext;

#[macro_use]
extern crate log;

mod config;
mod metrics;

pub use crate::config::KafkaConfig;

const MILLISECONDS_PER_SECOND: u64 = 1_000;
const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;

#[metric(
    name = "emitter/kafka/txbytes_uncompressed",
    description = "the number of bytes sent to the kafka channel before being zstd compressed"
)]
static TXBYTES_UNCOMPRESSED: Counter = Counter::new();

struct SharedContext {
    config: KafkaConfig,
    producer: FutureProducer<AugurProducerContext>,
    partitions: AtomicUsize,
}

impl SharedContext {
    fn new(
        config: KafkaConfig,
        producer: FutureProducer<AugurProducerContext>,
    ) -> anyhow::Result<Self> {
        let ctx = Self {
            config,
            producer,
            partitions: AtomicUsize::new(0),
        };

        ctx.update_partitions()?;

        Ok(ctx)
    }

    fn partitions(&self) -> usize {
        self.partitions.load(Ordering::Relaxed)
    }

    fn update_partitions(&self) -> anyhow::Result<()> {
        let metadata = self
            .producer
            .client()
            .fetch_metadata(
                Some(&self.config.topic),
                Timeout::After(Duration::from_secs(60)),
            )
            .context("Failed to fetch kafka topic metadata")?;

        match metadata
            .topics()
            .iter()
            .find(|topic| topic.name() == self.config.topic)
        {
            Some(topic) if topic.partitions().is_empty() => {
                Err(anyhow!("Kafka topic '{}' had no partitions", topic.name()))
            }
            Some(topic) => {
                self.partitions
                    .store(topic.partitions().len(), Ordering::Relaxed);
                Ok(())
            }
            None => Err(anyhow::anyhow!(
                "Unable to find partitions for topic '{}'",
                self.config.topic
            )),
        }
    }
}

pub struct KafkaEmitter<E> {
    shared: Arc<SharedContext>,
    encoder: Mutex<E>,
    task: JoinHandle<()>,
    pkey: usize,
}

impl<E: Encoder> KafkaEmitter<E> {
    pub fn new(config: KafkaConfig, encoder: E) -> anyhow::Result<Self> {
        let producer =
            FutureProducer::from_config_and_context(&config.client_config()?, Default::default())
                .context("Failed to initialize kafka producer")?;

        let shared = Arc::new(SharedContext::new(config, producer)?);
        let task = tokio::spawn(background_task(Arc::clone(&shared)));
        let pkey = hash(
            &nix::unistd::gethostname()
                .context("Failed to read current hostname")
                .context("Unable to determine kafka partitioning key")?,
        ) as usize;

        Ok(Self {
            shared,
            task,
            pkey,
            encoder: Mutex::new(encoder),
        })
    }

    pub async fn emit(&self, sample: Sample) -> anyhow::Result<()> {
        let partition = self.pkey.checked_rem(self.shared.partitions()).unwrap_or(0);

        let message = match self.encoder.lock().await.encode(sample)? {
            Some(message) => message,
            None => return Ok(()),
        };

        let duration = message
            .timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp = duration.as_secs() * MILLISECONDS_PER_SECOND
            + (duration.subsec_nanos() as u64) / (NANOSECONDS_PER_SECOND / MILLISECONDS_PER_SECOND);

        let record = FutureRecord::to(&self.shared.config.topic)
            .key(&())
            .partition(partition as i32)
            .timestamp(timestamp as i64)
            .payload(&message.data);

        self.shared
            .producer
            .send(record, Timeout::After(Duration::from_secs(60)))
            .await
            .map_err(|(e, _)| e)
            .context("Failed to send sample to the kafka broker")?;

        Ok(())
    }
}

async fn background_task(ctx: Arc<SharedContext>) {
    // Default interval between requests is 1h
    const INTERVAL_BASE: u64 = 3600;
    const INTERVAL_JITTER: u64 = 600;

    loop {
        let now = tokio::time::Instant::now();
        let ctx = Arc::clone(&ctx);
        if let Err(e) = tokio::task::spawn_blocking(move || ctx.update_partitions()).await {
            error!("Failed to update kafka partition count: {e}");
        };

        // Spread the requests from augur hosts over a 10 minute time range.
        // This should be enough to prevent augur instances started in a
        // similar time frame from overloading the kafka brokers. Over time
        // the requests will get smeared out across the entire hour.
        let duration = rand::thread_rng().gen_range(INTERVAL_BASE..INTERVAL_BASE + INTERVAL_JITTER);
        tokio::time::sleep_until(now + Duration::from_secs(duration)).await;
    }
}

#[async_trait::async_trait]
impl<E> Emitter for KafkaEmitter<E>
where
    E: Encoder + Send,
{
    async fn emit_sample(&self, sample: Sample) -> anyhow::Result<()> {
        self.emit(sample).await
    }
}

impl<E> Drop for KafkaEmitter<E> {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn hash<H: std::hash::Hash>(value: &H) -> u64 {
    use std::hash::Hasher;

    use fxhash::FxHasher64;

    let mut hasher = FxHasher64::default();
    value.hash(&mut hasher);
    hasher.finish()
}
