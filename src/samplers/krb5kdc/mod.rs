// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_trait::async_trait;

#[cfg(feature = "bpf")]
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::common::bpf::*;
use crate::config::SamplerConfig;
use crate::samplers::{Common, Sampler};

#[cfg(feature = "bpf")]
use crate::common::bpf::bpf_hash_char_to_map;
#[cfg(feature = "bpf")]
use std::collections::HashMap;

mod config;
mod stat;

pub use config::Krb5kdcConfig;
pub use stat::Krb5kdcStatistic;

#[allow(dead_code)]
pub struct Krb5kdc {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
    statistics: Vec<Krb5kdcStatistic>,
    path: String,
}

impl Krb5kdc {
    fn init_bpf(&mut self) -> Result<(), anyhow::Error> {
        #[cfg(feature = "bpf")]
        {
            let code = include_str!("bpf.c");
            let mut bpf = bcc::BPF::new(code)?;

            // collect the set of probes required from the statistics enabled.
            let mut probes = HashSet::new();
            for statistic in &self.statistics {
                for probe in statistic.bpf_probes_required(self.path.clone()) {
                    probes.insert(probe);
                }
            }

            // load + attach the kernel probes that are required to the bpf instance.
            for probe in probes {
                if let Err(err) = probe.try_attach_to_bpf(&mut bpf) {
                    if self.common.config().fault_tolerant() {
                        warn!(
                            "krb5kdc unable to attach probe to function {}",
                            probe.name.as_str()
                        );
                    } else {
                        Err(err)?;
                    }
                }
            }

            self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
        }
        Ok(())
    }
}

#[async_trait]
impl Sampler for Krb5kdc {
    type Statistic = Krb5kdcStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let fault_tolerant = common.config.general().fault_tolerant();
        let statistics = common.config().samplers().krb5kdc().statistics();
        let path = common.config().samplers().krb5kdc().path();
        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
            statistics,
            path,
        };

        if let Err(e) = sampler.init_bpf() {
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
        if common.config().samplers().krb5kdc().enabled() {
            match Self::new(common.clone()) {
                Ok(mut sampler) => {
                    common.runtime().spawn(async move {
                        loop {
                            let _ = sampler.sample().await;
                        }
                    });
                }
                Err(e) => {
                    if !common.config.fault_tolerant() {
                        fatal!("failed to initialize krb5kdc sampler {}", e);
                    } else {
                        error!("failed to initialize krb5kdc sampler {}", e);
                    }
                }
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
        self.common.config().samplers().krb5kdc()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }

        #[cfg(feature = "bpf")]
        if let Some(ref bpf) = self.bpf {
            let bpf = bpf.lock().unwrap();
            let mut table_map = HashMap::new();

            table_map.insert(
                "counts_finish_process_as_req",
                bpf_hash_char_to_map(
                    &(*bpf)
                        .inner
                        .table("counts_finish_process_as_req")
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
                ),
            );

            table_map.insert(
                "counts_finish_dispatch_cache",
                bpf_hash_char_to_map(
                    &(*bpf)
                        .inner
                        .table("counts_finish_dispatch_cache")
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
                ),
            );

            table_map.insert(
                "counts_process_tgs_req",
                bpf_hash_char_to_map(
                    &(*bpf)
                        .inner
                        .table("counts_process_tgs_req")
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
                ),
            );

            for stat in self.statistics.iter() {
                if let Some(entry_map) = table_map.get(stat.bpf_table()) {
                    let val = entry_map.get(stat.bpf_entry()).unwrap_or(&0);
                    self.metrics()
                        .record_counter(stat, Instant::now(), *val)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                } else {
                    self.metrics()
                        .record_counter(stat, Instant::now(), 0)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                }
            }
        }
        Ok(())
    }
}
