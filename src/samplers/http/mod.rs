// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::io::{Error, ErrorKind};
use std::time::*;

use async_trait::async_trait;
use hyper::client::ResponseFuture;
use hyper::{Body, Uri};
use rustcommon_metrics::*;

use crate::config::*;
use crate::samplers::Common;
use crate::Sampler;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

pub enum Client {
    Http(hyper::Client<hyper::client::HttpConnector>),
    Https(hyper::Client<hyper_boring::HttpsConnector<hyper::client::HttpConnector>>),
}

impl Client {
    fn get(&self, uri: Uri) -> ResponseFuture {
        match self {
            Client::Http(client) => client.get(uri),
            Client::Https(client) => client.get(uri),
        }
    }
}

pub struct Http {
    client: Client,
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
            return Err(format_err!("no url configured for http sampler"));
        }
        let url = url.unwrap();
        let protocol: Vec<&str> = url.split(':').collect();
        if protocol.len() < 2 {
            return Err(format_err!("invalid url"));
        }
        let client = match protocol[0] {
            "http" => Client::Http(
                hyper::Client::builder()
                    .pool_max_idle_per_host(1)
                    .build::<_, Body>(hyper::client::HttpConnector::new()),
            ),
            "https" => Client::Https(
                hyper::Client::builder()
                    .pool_max_idle_per_host(1)
                    .build::<_, Body>(hyper_boring::HttpsConnector::new().unwrap()),
            ),
            _ => {
                return Err(format_err!(
                    "invalid protocol for url: {}",
                    common.config.samplers().http().url().unwrap()
                ));
            }
        };
        let url = common.config.samplers().http().url();
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
        let response = self
            .client
            .get(self.url.as_ref().unwrap().parse::<Uri>().unwrap())
            .await
            .unwrap();

        if response.status().is_success() {
            let body = hyper::body::to_bytes(response).await.unwrap();
            if let Ok(json) = json::parse(std::str::from_utf8(&body).unwrap_or("")) {
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
            Err(Error::new(ErrorKind::Other, "http request failed!"))
        }
    }
}
