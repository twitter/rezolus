// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use rustcommon_metrics::*;

use crate::common::bpf::*;
use crate::common::*;
use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::Sampler;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Ext4 {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
}

#[async_trait]
impl Sampler for Ext4 {
    type Statistic = Ext4Statistic;
    fn new(common: Common) -> Result<Self, failure::Error> {
        let fault_tolerant = common.config.general().fault_tolerant();

        #[allow(unused_mut)]
        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
        };

        if let Err(e) = sampler.initialize_bpf() {
            if !fault_tolerant {
                return Err(e);
            }
        }

        Ok(sampler)
    }

    fn spawn(common: Common) {
        if let Ok(mut sampler) = Self::new(common.clone()) {
            common.handle.spawn(async move {
                loop {
                    let _ = sampler.sample().await;
                }
            });
        } else if !common.config.fault_tolerant() {
            fatal!("failed to initialize ext4 sampler");
        } else {
            error!("failed to initialize ext4 sampler");
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().ext4()
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
        self.map_result(self.sample_bpf())?;

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

impl Ext4 {
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
            if self.enabled() && self.bpf_enabled() {
                debug!("initializing bpf");
                // load the code and compile
                let code = include_str!("bpf.c").to_string();
                let addr = "0x".to_string()
                    + &crate::common::bpf::symbol_lookup("ext4_file_operations").unwrap();
                let code = code.replace("EXT4_FILE_OPERATIONS", &addr);
                let mut bpf = bcc::core::BPF::new(&code)?;

                // load + attach kprobes!
                let generic_file_read_iter_entry = bpf.load_kprobe("trace_read_entry")?;
                let ext4_file_write_iter_entry = bpf.load_kprobe("trace_entry")?;
                let ext4_file_open_entry = bpf.load_kprobe("trace_entry")?;
                let ext4_sync_file_entry = bpf.load_kprobe("trace_entry")?;
                let generic_file_read_iter_return = bpf.load_kprobe("trace_read_return")?;
                let ext4_file_write_iter_return = bpf.load_kprobe("trace_write_return")?;
                let ext4_file_open_return = bpf.load_kprobe("trace_open_return")?;
                let ext4_sync_file_return = bpf.load_kprobe("trace_fsync_return")?;

                bpf.attach_kprobe("generic_file_read_iter", generic_file_read_iter_entry)?;
                bpf.attach_kprobe("ext4_file_write_iter", ext4_file_write_iter_entry)?;
                bpf.attach_kprobe("ext4_file_open", ext4_file_open_entry)?;
                bpf.attach_kprobe("ext4_sync_file", ext4_sync_file_entry)?;
                bpf.attach_kretprobe("generic_file_read_iter", generic_file_read_iter_return)?;
                bpf.attach_kretprobe("ext4_file_write_iter", ext4_file_write_iter_return)?;
                bpf.attach_kretprobe("ext4_file_open", ext4_file_open_return)?;
                bpf.attach_kretprobe("ext4_sync_file", ext4_sync_file_return)?;

                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
            }
        }

        Ok(())
    }

    #[cfg(feature = "bpf")]
    fn sample_bpf(&self) -> Result<(), std::io::Error> {
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
        Ok(())
    }
}
