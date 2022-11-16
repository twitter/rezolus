use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use augur_annotate as annotate;
use augur_collector_perf::{PerfCollector, PerfConfig};
use augur_common::{Annotator, Emitter, Sample};
use augur_emitter_pyroscope::PyroscopeEmitter;
use config::DebugMode;
use rustcommon_metrics::{metric, Counter};

#[macro_use]
extern crate log;

mod config;

pub use self::config::Config;

#[metric(name = "samples")]
static TOTAL_SAMPLES: Counter = Counter::new();

#[metric(
    name = "samples_success",
    description = "count of samples that were successfully processed"
)]
static SAMPLES_SUCCESS: Counter = Counter::new();

#[metric(
    name = "samples_error",
    description = "count of samples for which processing failed with an error"
)]
static SAMPLES_ERROR: Counter = Counter::new();

/// All annotators used in augur
struct Annotate {
    command: annotate::Command,
    hostname: annotate::Hostname,
    systemd: annotate::Systemd,
    mesos: annotate::Mesos,
}

impl Annotate {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            command: annotate::Command::new(),
            hostname: annotate::Hostname::new().context("Unable to create hostname annotator")?,
            systemd: annotate::Systemd::new().context("Unable to create systemd annotator")?,
            mesos: annotate::Mesos::new(),
        })
    }
}

pub struct Profiler {
    config: Config,
    perf: PerfCollector,
    anno: Arc<Annotate>,
    emitter: Arc<dyn Emitter>,
}

impl Profiler {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let perf_config = PerfConfig {
            frequency: config.general.frequency,
            period: config.general.period,
        };
        let perf =
            PerfCollector::new(perf_config).context("while setting up the perf collector")?;

        // let kafka = Arc::new(
        //     KafkaEmitter::new(config.kafka.clone(), ZstdThriftEncoder::default())
        //         .context("Failed to create kafka emitter")?,
        // );

        let pyroscope = Arc::new(
            PyroscopeEmitter::new(augur_emitter_pyroscope::Config {
                upstream: "smf1-gcj-09-sr1.prod.twitter.com".to_string(),
                batch_time: Duration::from_secs(10),
            })
            .context("Failed to create pyroscope emitter")?,
        );

        let anno = Arc::new(Annotate::new()?);

        Ok(Self {
            config,
            anno,
            perf,
            emitter: pyroscope,
        })
    }

    async fn process_sample(
        mut sample: Sample,
        mode: DebugMode,
        anno: Arc<Annotate>,
        kafka: Arc<dyn Emitter>,
    ) -> anyhow::Result<()> {
        anno.command.annotate(&mut sample).await;
        anno.hostname.annotate(&mut sample).await;
        anno.systemd.annotate(&mut sample).await;
        anno.mesos.annotate(&mut sample).await;

        match mode {
            DebugMode::Production => kafka
                .emit_sample(sample)
                .await
                .context("Failed to emit sample")?,
            DebugMode::Terminal => {
                let mut writer = std::io::stdout().lock();

                serde_json::to_writer(&mut writer, &sample)
                    .context("Unable to write sample to stdout")?;
                write!(writer, "\n").context("Unable to write sample to stdout")?;
            }
            DebugMode::Quiet => {}
        }

        Ok(())
    }

    pub async fn profile(&mut self) -> anyhow::Result<()> {
        loop {
            let sample = self
                .perf
                .next_event()
                .await
                .context("While gathering the next stack sample")?;

            let mode = self.config.general.debug;
            let anno = Arc::clone(&self.anno);
            let emitter = Arc::clone(&self.emitter);

            tokio::spawn(async move {
                TOTAL_SAMPLES.add(1);

                match Self::process_sample(sample, mode, anno, emitter).await {
                    Ok(_) => SAMPLES_SUCCESS.increment(),
                    Err(e) => {
                        warn!("An error occurred during sample processing: {e:#}");
                        SAMPLES_ERROR.increment()
                    }
                }
            });
        }
    }
}
