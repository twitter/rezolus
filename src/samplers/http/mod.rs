// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::io::{Error, ErrorKind};

use crate::metrics::{Output, Summary};
use async_trait::async_trait;

use crate::config::*;
use crate::samplers::Common;
use crate::*;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

pub struct Http {
    client: reqwest::blocking::Client,
    common: Common,
    passthrough: bool,
    url: Option<String>,
}

#[async_trait]
impl Sampler for Http {
    type Statistic = HttpStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let url = common.config.samplers().http().url();
        let passthrough = common.config.samplers().http().passthrough();
        if url.is_none() && common.config.samplers().http().enabled() {
            return Err(format_err!("no http url configured"));
        }
        let client = reqwest::blocking::Client::new();
        let ret = Self {
            client,
            common,
            passthrough,
            url,
        };
        Ok(ret)
    }

    fn spawn(common: Common) {
        if common.config().samplers().http().enabled() {
            if let Ok(mut sampler) = Self::new(common.clone()) {
                common.runtime().spawn(async move {
                    loop {
                        let _ = sampler.sample().await;
                    }
                });
            } else if !common.config.fault_tolerant() {
                fatal!("failed to initialize http sampler");
            } else {
                error!("failed to initialize http sampler");
            }
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().http()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }

        if self.url.is_none() {
            return Err(Error::new(
                ErrorKind::Other,
                "no url configured for http sampler",
            ));
        }

        let time = Instant::now();
        if let Ok(response) = self.client.get(self.url.as_ref().unwrap()).send() {
            if let Ok(body) = response.text() {
                if let Ok(json) = json::parse(&body) {
                    let mut statistics = std::collections::HashMap::new();
                    for counter in self.common.config().samplers().http().counters() {
                        statistics.insert(
                            counter.to_string(),
                            HttpStatistic::new(counter.to_string(), Source::Counter),
                        );
                    }
                    for gauge in self.common.config().samplers().http().gauges() {
                        statistics.insert(
                            gauge.to_string(),
                            HttpStatistic::new(gauge.to_string(), Source::Counter),
                        );
                    }
                    for (key, value) in json.entries() {
                        if let Some(value) = value.as_u64() {
                            if let Some(statistic) = statistics.get(key) {
                                self.common().metrics().register(statistic);
                                self.common()
                                    .metrics()
                                    .add_summary(statistic, Summary::stream(self.samples()));
                                if self.passthrough {
                                    self.common()
                                        .metrics()
                                        .add_output(statistic, Output::Reading);
                                }
                                for percentile in self.sampler_config().percentiles() {
                                    self.common()
                                        .metrics()
                                        .add_output(statistic, Output::Percentile(*percentile));
                                }
                                match statistic.source() {
                                    Source::Counter => {
                                        let _ = self
                                            .common()
                                            .metrics()
                                            .record_counter(statistic, time, value);
                                    }
                                    Source::Gauge => {
                                        let _ = self
                                            .common()
                                            .metrics()
                                            .record_gauge(statistic, time, value);
                                    }
                                    _ => unimplemented!(),
                                }
                            } else if self.passthrough {
                                let statistic = HttpStatistic::new(key.to_string(), Source::Gauge);
                                self.common().metrics().register(&statistic);
                                self.common()
                                    .metrics()
                                    .add_output(&statistic, Output::Reading);
                                let _ = self
                                    .common()
                                    .metrics()
                                    .record_gauge(&statistic, time, value);
                            }
                        }
                    }
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "failed to parse response as json!",
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "failed to read response body!",
                ))
            }
        } else {
            Err(Error::new(ErrorKind::Other, "http request failed!"))
        }
    }
}
