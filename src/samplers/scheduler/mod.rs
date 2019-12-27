// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::bpf::*;
use crate::config::{Config, SamplerConfig};
use crate::samplers::Common;
use crate::Sampler;
use async_trait::async_trait;
#[cfg(feature = "ebpf")]
use bcc;
use metrics::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use tokio::runtime::Handle;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Scheduler {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
}

#[async_trait]
impl Sampler for Scheduler {
    type Statistic = SchedulerStatistic;
    fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Result<Self, failure::Error> {
        #[cfg(feature = "ebpf")]
        let bpf = if config.disk().ebpf() {
            debug!("initializing ebpf");
            // load the code and compile
            let code = include_str!("bpf.c");
            let mut bpf = bcc::core::BPF::new(code)?;

            // load + attach kprobes!
            let trace_run = bpf.load_kprobe("trace_run")?;
            let trace_ttwu_do_wakeup = bpf.load_kprobe("trace_ttwu_do_wakeup")?;
            let trace_wake_up_new_task = bpf.load_kprobe("trace_wake_up_new_task")?;

            bpf.attach_kprobe("finish_task_switch", trace_run)?;
            bpf.attach_kprobe("wake_up_new_task", trace_wake_up_new_task)?;
            bpf.attach_kprobe("ttwu_do_wakeup", trace_ttwu_do_wakeup)?;

            Some(Arc::new(Mutex::new(BPF { inner: bpf })))
        } else {
            None
        };

        #[cfg(not(feature = "ebpf"))]
        let bpf = None;

        Ok(Self {
            bpf,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common: Common::new(config, metrics),
        })
    }

    fn spawn(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>, handle: &Handle) {
        if let Ok(mut sampler) = Self::new(config.clone(), metrics) {
            handle.spawn(async move {
                loop {
                    let _ = sampler.sample().await;
                }
            });
        } else if !config.fault_tolerant() {
            fatal!("failed to initialize sampler");
        } else {
            error!("failed to initialize sampler");
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().scheduler()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if !self.sampler_config().enabled() {
            if let Some(ref mut delay) = self.delay() {
                delay.tick().await;
            }

            return Ok(());
        }

        debug!("sampling");
        self.register();

        // sample ebpf
        #[cfg(feature = "ebpf")]
        {
            if self.bpf_last.lock().unwrap().elapsed() >= self.general_config().window() {
                if let Some(ref bpf) = self.bpf {
                    let bpf = bpf.lock().unwrap();
                    let time = time::precise_time_ns();
                    for statistic in self.sampler_config().statistics() {
                        if let Some(table) = statistic.ebpf_table() {
                            let mut table = (*bpf).inner.table(table);

                            for (&value, &count) in &map_from_table(&mut table) {
                                if count > 0 {
                                    self.metrics().record_distribution(
                                        statistic,
                                        time,
                                        value * 1000,
                                        count,
                                    );
                                }
                            }
                        }
                    }
                }
                *self.bpf_last.lock().unwrap() = Instant::now();
            }
        }

        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        Ok(())
    }

    fn summary(&self, _statistic: &Self::Statistic) -> Option<Summary> {
        Some(Summary::histogram(
            1_000_000,
            2,
            Some(self.general_config().window()),
        ))
    }
}
