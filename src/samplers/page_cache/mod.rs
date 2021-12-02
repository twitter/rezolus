// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
#[cfg(feature = "bpf")]
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

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
    common: Common,
    statistics: Vec<PageCacheStatistic>,
    counters: HashMap<PageCacheStatistic, u64>,
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
            common,
            statistics,
            counters: HashMap::new(),
        };

        if let Err(e) = sampler.initialize_bpf() {
            error!("{}", e);
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
        if common.config().samplers().page_cache().enabled() {
            if let Ok(mut interrupt) = PageCache::new(common.clone()) {
                common.runtime().spawn(async move {
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
        {
            let result = self.sample_bpf_counters();
            self.map_result(result)?;
        }

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

                // collect the set of probes required from the statistics enabled.
                let mut probes = HashSet::new();
                for statistic in &self.statistics {
                    for probe in statistic.bpf_probes_required() {
                        probes.insert(probe);
                    }
                }

                // load + attach the kernel probes that are required to the bpf instance.
                for probe in probes {
                    if self.common.config.fault_tolerant() {
                        let _ = probe.try_attach_to_bpf(&mut bpf);
                    } else {
                        probe.try_attach_to_bpf(&mut bpf)?;
                    }
                }

                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })))
            }
        }

        Ok(())
    }

    #[cfg(feature = "bpf")]
    fn sample_bpf_counters(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref bpf) = self.bpf {
            let bpf = bpf.lock().unwrap();
            let time = std::time::Instant::now();
            let mut page_accessed = 0;
            let mut buffer_dirty = 0;
            let mut add_to_page_cache_lru = 0;
            let mut page_dirtied = 0;

            // to make things simple for wraparound behavior, clear each BPF
            // counter after reading it.
            if let Ok(mut table) = (*bpf).inner.table("page_accessed") {
                page_accessed = crate::common::bpf::parse_u64(table.iter().next().unwrap().value);
                let _ = table.set(&mut [0, 0, 0, 0], &mut [0, 0, 0, 0, 0, 0, 0, 0]);
            }
            if let Ok(mut table) = (*bpf).inner.table("buffer_dirty") {
                buffer_dirty = crate::common::bpf::parse_u64(table.iter().next().unwrap().value);
                let _ = table.set(&mut [0, 0, 0, 0], &mut [0, 0, 0, 0, 0, 0, 0, 0]);
            }
            if let Ok(mut table) = (*bpf).inner.table("add_to_page_cache_lru") {
                add_to_page_cache_lru =
                    crate::common::bpf::parse_u64(table.iter().next().unwrap().value);
                let _ = table.set(&mut [0, 0, 0, 0], &mut [0, 0, 0, 0, 0, 0, 0, 0]);
            }
            if let Ok(mut table) = (*bpf).inner.table("page_dirtied") {
                page_dirtied = crate::common::bpf::parse_u64(table.iter().next().unwrap().value);
                let _ = table.set(&mut [0, 0, 0, 0], &mut [0, 0, 0, 0, 0, 0, 0, 0]);
            }

            // the logic here is taken from https://github.com/iovisor/bcc/blob/master/tools/cachestat.py
            let total = page_accessed.saturating_sub(buffer_dirty);
            let misses = add_to_page_cache_lru.saturating_sub(page_dirtied);

            // misses may be overestimated due to readahead adding more pages
            // than needed. If this is the case, assume misses = total,
            let misses = if misses > total { total } else { misses };
            let hits = total.saturating_sub(misses);

            if let Some(count) = self.counters.get_mut(&PageCacheStatistic::Hit) {
                *count += hits;
            } else {
                self.counters.insert(PageCacheStatistic::Hit, hits);
            }

            if let Some(count) = self.counters.get_mut(&PageCacheStatistic::Miss) {
                *count += misses;
            } else {
                self.counters.insert(PageCacheStatistic::Miss, misses);
            }

            let _ = self.metrics().record_counter(
                &PageCacheStatistic::Hit,
                time,
                *self.counters.get(&PageCacheStatistic::Hit).unwrap_or(&0),
            );
            let _ = self.metrics().record_counter(
                &PageCacheStatistic::Miss,
                time,
                *self.counters.get(&PageCacheStatistic::Miss).unwrap_or(&0),
            );
        }
        Ok(())
    }
}
