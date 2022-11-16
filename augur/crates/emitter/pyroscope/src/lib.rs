//! Emitter that exports pprof profiles to a pyroscope instance.

use std::time::{Duration, SystemTime};

use anyhow::Context;
use augur_common::{Emitter, Sample};
use augur_pprof::{proto, ProfileBuilder};
use protobuf::Message;
use reqwest::Url;
use rustcommon_metrics::{metric, Counter};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[macro_use]
extern crate log;

const NANOSECONDS_PER_SECOND: i64 = 1_000_000_000;

#[metric(
    name = "emitter/pyroscope/post_success",
    description = "the number of successful profiles that have been posted to the pyroscope /ingest endpoint"
)]
static POST_SUCCESS: Counter = Counter::new();

#[serde_with::serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    // URL to the pyroscope instance
    pub upstream: String,

    // Length of time for which samples will be batched up before sending them
    // to the pyroscope server.
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub batch_time: Duration,
}

pub struct PyroscopeEmitter {
    client: reqwest::Client,
    state: Mutex<State>,
    ingest_url: Url,
}

impl PyroscopeEmitter {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let state = State::new(config.batch_time);
        let client = reqwest::Client::builder()
            .user_agent("augur-agent/0.1.0")
            .build()?;

        Ok(Self {
            client,
            state: Mutex::new(state),
            ingest_url: Url::parse(&format!("http://{}/ingest", config.upstream))?,
        })
    }

    pub async fn emit(&self, sample: Sample) -> anyhow::Result<()> {
        let mut profile = match self.state.lock().await.add(sample) {
            Some(profile) => profile,
            None => return Ok(()),
        };

        let start = profile.time_nanos / NANOSECONDS_PER_SECOND;
        let end = (profile.time_nanos + profile.duration_nanos) / NANOSECONDS_PER_SECOND;

        // Sample weight is in µs so we say that the sampling period is 1µs.
        profile.period = 1;

        debug!(
            "Uploading profile to {}, time range: {start}-{end}",
            self.ingest_url
        );

        let mut data = Vec::new();
        profile.write_to_vec(&mut data)?;

        let response = self
            .client
            .post(self.ingest_url.clone())
            .query(&[
                ("name", "augur"),
                ("format", "pprof"),
                ("spyName", "augur"),
                ("from", &start.to_string()),
                ("until", &end.to_string()),
            ])
            .body(data)
            .send()
            .await
            .context("Failed to POST profile to pyroscope /ingest endpoint")?;

        let status = response.status();
        if response.status().is_success() {
            POST_SUCCESS.increment();
            return Ok(());
        }

        let body = response.text().await?;

        error!("POST to pyroscope /ingest returned {status}: {body}");

        Ok(())
    }
}

struct State {
    builder: ProfileBuilder,
    batch_start: Option<SystemTime>,
    /// How long each batch is supposed to be
    batch_duration: Duration,
}

impl State {
    pub fn new(batch_time: Duration) -> Self {
        Self {
            builder: ProfileBuilder::new(),
            batch_start: None,
            batch_duration: batch_time,
        }
    }

    pub fn add(&mut self, sample: Sample) -> Option<proto::Profile> {
        let start = *self.batch_start.get_or_insert(sample.time);
        let duration = sample
            .time
            .duration_since(start)
            .unwrap_or(Duration::from_secs(0));

        self.builder.add(sample);

        if duration >= self.batch_duration {
            self.batch_start = None;

            let len = self.builder.len();
            let profile = std::mem::take(&mut self.builder).build();

            debug!("Emitting a profile with {len} samples");

            Some(profile)
        } else {
            None
        }
    }
}

#[async_trait::async_trait]
impl Emitter for PyroscopeEmitter {
    async fn emit_sample(&self, sample: Sample) -> anyhow::Result<()> {
        self.emit(sample).await
    }
}
