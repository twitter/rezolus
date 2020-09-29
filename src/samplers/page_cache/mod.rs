// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::{Arc, Mutex};
use std::time::*;

use async_trait::async_trait;

use crate::common::bpf::*;
use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::Sampler;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct PageCache {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
    statistics: Vec<PageCacheStatistic>,
}

#[async_trait]
impl Sampler for PageCache {
    type Statistic = PageCacheStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let fault_tolerant = common.config.general().fault_tolerant();
        let statistics = common.config().samplers().page_cache().statistics();

        #[allow(unused_mut)]
        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
            statistics,
        };

        if let Err(e) = sampler.initialize_bpf() {
            if !fault_tolerant {
                return Err(e);
            }
        }

        if sampler.sampler_config().enabled() {
            sampler.register();
        }

        Ok(sampler)
    }

    fn spawn(common: Common) {
        if let Ok(mut interrupt) = PageCache::new(common.clone()) {
            common.handle.spawn(async move {
                loop {
                    let _ = interrupt.sample().await;
                }
            });
        } else if !common.config.fault_tolerant() {
            fatal!("failed to initialize page_cache sampler");
        } else {
            error!("failed to initialize page_cache sampler");
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().page_cache()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }

        debug!("sampling");

        #[cfg(feature = "bpf")]
        self.map_result(self.sample_bpf())?;

        Ok(())
    }
}

impl PageCache {
    #[cfg(feature = "bpf")]
    fn bpf_enabled(&self) -> bool {
        if self.sampler_config().bpf() {
            for statistic in &self.statistics {
                if statistic.is_bpf() {
                    return true;
                }
            }
        }
        false
    }

    fn initialize_bpf(&mut self) -> Result<(), anyhow::Error> {
        #[cfg(feature = "bpf")]
        {
            if self.enabled() && self.bpf_enabled() {
                debug!("initializing bpf");

                let code = include_str!("bpf.c");
                let mut bpf = bcc::BPF::new(code)?;

                bcc::Kprobe::new()
                    .handler("trace_mark_page_accessed")
                    .function("mark_page_accessed")
                    .attach(&mut bpf)?;
                bcc::Kprobe::new()
                    .handler("trace_mark_buffer_dirty")
                    .function("mark_buffer_dirty")
                    .attach(&mut bpf)?;
                bcc::Kprobe::new()
                    .handler("trace_add_to_page_cache_lru")
                    .function("add_to_page_cache_lru")
                    .attach(&mut bpf)?;
                bcc::Kprobe::new()
                    .handler("trace_account_page_dirtied")
                    .function("account_page_dirtied")
                    .attach(&mut bpf)?;

                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })))
            }
        }

        Ok(())
    }

    #[cfg(feature = "bpf")]
    fn sample_bpf(&self) -> Result<(), std::io::Error> {
        if self.bpf_last.lock().unwrap().elapsed()
            >= Duration::new(self.general_config().window() as u64, 0)
        {
            if let Some(ref bpf) = self.bpf {
                let bpf = bpf.lock().unwrap();
                let time = Instant::now();
                let mut page_accessed = 0;
                let mut buffer_dirty = 0;
                let mut add_to_page_cache_lru = 0;
                let mut page_dirtied = 0;
                if let Ok(table) = (*bpf).inner.table("page_accessed") {
                    page_accessed =
                        crate::common::bpf::parse_u64(table.iter().next().unwrap().value);
                }
                if let Ok(table) = (*bpf).inner.table("buffer_dirty") {
                    buffer_dirty =
                        crate::common::bpf::parse_u64(table.iter().next().unwrap().value);
                }
                if let Ok(table) = (*bpf).inner.table("add_to_page_cache_lru") {
                    add_to_page_cache_lru =
                        crate::common::bpf::parse_u64(table.iter().next().unwrap().value);
                }
                if let Ok(table) = (*bpf).inner.table("page_dirtied") {
                    page_dirtied =
                        crate::common::bpf::parse_u64(table.iter().next().unwrap().value);
                }
                let total = page_accessed.wrapping_sub(buffer_dirty);
                let misses = add_to_page_cache_lru.wrapping_sub(page_dirtied);
                let hits = total.wrapping_sub(misses);
                let _ = self
                    .metrics()
                    .record_counter(&PageCacheStatistic::Hit, time, hits);
                let _ = self
                    .metrics()
                    .record_counter(&PageCacheStatistic::Miss, time, misses);
            }
            *self.bpf_last.lock().unwrap() = Instant::now();
        }
        Ok(())
    }
}
