// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use atomics::AtomicU32;
#[cfg(feature = "bpf")]
use bcc;
use metrics::*;
use tokio::runtime::Handle;

use crate::common::bpf::*;
use crate::common::*;
use crate::config::{Config, SamplerConfig};
use crate::samplers::Common;
use crate::Sampler;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Xfs {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
}

#[async_trait]
impl Sampler for Xfs {
    type Statistic = XfsStatistic;
    fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Result<Self, failure::Error> {
        let fault_tolerant = config.general().fault_tolerant();

        #[allow(unused_mut)]
        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common: Common::new(config, metrics),
        };

        if let Err(e) = sampler.initialize_bpf() {
            if !fault_tolerant {
                return Err(e);
            }
        }

        Ok(sampler)
    }

    fn spawn(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>, handle: &Handle) {
        if let Ok(mut sampler) = Self::new(config.clone(), metrics) {
            handle.spawn(async move {
                loop {
                    let _ = sampler.sample().await;
                }
            });
        } else if !config.fault_tolerant() {
            fatal!("failed to initialize xfs sampler");
        } else {
            error!("failed to initialize xfs sampler");
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().xfs()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }

        debug!("sampling");
        self.register();

        // sample bpf
        #[cfg(feature = "bpf")]
        {
            if self.bpf_last.lock().unwrap().elapsed() >= self.general_config().window() {
                if let Some(ref bpf) = self.bpf {
                    let bpf = bpf.lock().unwrap();
                    let time = time::precise_time_ns();
                    for statistic in self.sampler_config().statistics() {
                        if let Some(table) = statistic.bpf_table() {
                            let mut table = (*bpf).inner.table(table);

                            for (&value, &count) in &map_from_table(&mut table) {
                                if count > 0 {
                                    self.metrics().record_distribution(
                                        statistic,
                                        time,
                                        value * MICROSECOND,
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

        Ok(())
    }

    fn summary(&self, _statistic: &Self::Statistic) -> Option<Summary> {
        Some(Summary::histogram(
            SECOND,
            2,
            Some(self.general_config().window()),
        ))
    }
}

impl Xfs {
    // checks that bpf is enabled in config and one or more bpf stats enabled
    #[cfg(feature = "bpf")]
    fn bpf_enabled(&self) -> bool {
        if self.sampler_config().bpf() {
            for statistic in self.sampler_config().statistics() {
                if statistic.bpf_table().is_some() {
                    return true;
                }
            }
        }
        false
    }

    fn initialize_bpf(&mut self) -> Result<(), failure::Error> {
        #[cfg(feature = "bpf")]
        {
            if self.bpf_enabled() {
                debug!("initializing bpf");

                // load the code and compile
                let code = include_str!("bpf.c");
                let mut bpf = bcc::core::BPF::new(code)?;

                // load + attach kprobes!
                let read_entry = bpf.load_kprobe("trace_entry")?;
                let write_entry = bpf.load_kprobe("trace_entry")?;
                let open_entry = bpf.load_kprobe("trace_entry")?;
                let fsync_entry = bpf.load_kprobe("trace_entry")?;
                let read_return = bpf.load_kprobe("trace_read_return")?;
                let write_return = bpf.load_kprobe("trace_write_return")?;
                let open_return = bpf.load_kprobe("trace_open_return")?;
                let fsync_return = bpf.load_kprobe("trace_fsync_return")?;

                bpf.attach_kprobe("xfs_file_read_iter", read_entry)?;
                bpf.attach_kprobe("xfs_file_write_iter", write_entry)?;
                bpf.attach_kprobe("xfs_file_open", open_entry)?;
                bpf.attach_kprobe("xfs_file_fsync", fsync_entry)?;
                bpf.attach_kretprobe("xfs_file_read_iter", read_return)?;
                bpf.attach_kretprobe("xfs_file_write_iter", write_return)?;
                bpf.attach_kretprobe("xfs_file_open", open_return)?;
                bpf.attach_kretprobe("xfs_file_fsync", fsync_return)?;

                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
            }
        }

        Ok(())
    }
}
