use async_trait::async_trait;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::common::bpf::BPF;
use crate::config::SamplerConfig;
use crate::samplers::{Common, Sampler};

mod config;
mod stat;

pub use config::LibCallConfig;
pub use stat::LibCallStatistic;

#[allow(dead_code)]
pub struct LibCall {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
    statistics: Vec<LibCallStatistic>,
}

#[async_trait]
impl Sampler for LibCall {
    type Statistic = LibCallStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
            statistics: vec![LibCallStatistic::FooBar],
        };
        if sampler.sampler_config().enabled() {
            info!("We're registering libcall");
            sampler.register();
        }
        info!("We're newing libcall");
        Ok(sampler)
    }

    fn spawn(common: Common) {
        if common.config().samplers().libcall().enabled() {
            info!("We're spawning libcall");
            if let Ok(mut sampler) = Self::new(common.clone()) {
                common.runtime().spawn(async move {
                    loop {
                        let _ = sampler.sample().await;
                    }
                });
            } else if !common.config.fault_tolerant() {
                fatal!("failed to initialize libcall sampler");
            } else {
                error!("failed to initialize libcall sampler");
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
        self.common.config().samplers().libcall()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }
        for statistic in self.statistics.iter() {
            info!("Recording statistic: {:?}", statistic);
            let _ = self.metrics().record_counter(statistic, Instant::now(), 1);
        }

        info!("Sampling libcall!");

        Ok(())
    }
}
