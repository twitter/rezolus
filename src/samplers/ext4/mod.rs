// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "bpf")]
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::common::bpf::*;
use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::*;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Ext4 {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
    statistics: Vec<Ext4Statistic>,
}

#[async_trait]
impl Sampler for Ext4 {
    type Statistic = Ext4Statistic;
    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let fault_tolerant = common.config.general().fault_tolerant();
        let statistics = common.config().samplers().ext4().statistics();

        #[allow(unused_mut)]
        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
            statistics,
        };

        if let Err(e) = sampler.initialize_bpf() {
            error!("failed to initialize bpf: {}", e);
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
        if common.config().samplers().ext4().enabled() {
            if let Ok(mut sampler) = Self::new(common.clone()) {
                common.runtime().spawn(async move {
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

        // sample bpf
        #[cfg(feature = "bpf")]
        self.map_result(self.sample_bpf())?;

        Ok(())
    }
}

impl Ext4 {
    // checks that bpf is enabled in config and one or more bpf stats enabled
    #[cfg(feature = "bpf")]
    fn bpf_enabled(&self) -> bool {
        if self.sampler_config().bpf() {
            for statistic in &self.statistics {
                if statistic.bpf_table().is_some() {
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
                // load the code and compile
                let code = include_str!("bpf.c").to_string();
                let addr = "0x".to_string()
                    + &crate::common::bpf::symbol_lookup("ext4_file_operations").unwrap();
                let code = code.replace("EXT4_FILE_OPERATIONS", &addr);
                let code = code.replace(
                    "VALUE_TO_INDEX2_FUNC",
                    include_str!("../../common/value_to_index2.c"),
                );
                let mut bpf = bcc::BPF::new(&code)?;

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
                        if let Err(e) = probe.try_attach_to_bpf(&mut bpf) {
                            warn!("skipping {} with error: {}", probe.name, e);
                        }
                    } else {
                        probe.try_attach_to_bpf(&mut bpf)?;
                    }
                }

                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
            }
        }

        Ok(())
    }

    #[cfg(feature = "bpf")]
    fn sample_bpf(&self) -> Result<(), std::io::Error> {
        if self.bpf_last.lock().unwrap().elapsed()
            >= Duration::from_secs(self.general_config().window() as u64)
        {
            if let Some(ref bpf) = self.bpf {
                let bpf = bpf.lock().unwrap();
                let time = Instant::now();
                for statistic in self.statistics.iter().filter(|s| s.bpf_table().is_some()) {
                    if let Ok(mut table) = (*bpf).inner.table(statistic.bpf_table().unwrap()) {
                        for (&value, &count) in &map_from_table(&mut table) {
                            if count > 0 {
                                let _ = self.metrics().record_bucket(
                                    statistic,
                                    time,
                                    value * crate::MICROSECOND,
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
