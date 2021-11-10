// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_trait::async_trait;

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

            // define the kernel probes here.
            let mut probes = Probes::new();
            probes.add_user_probe(
                String::from("finish_process_as_req"),
                String::from("count_finish_process_as_req"),
                ProbeLocation::Entry,
                self.path.clone(),
                [
                    Krb5kdcStatistic::FinishProcessAsReqUnknown,
                    Krb5kdcStatistic::FinishProcessAsReqNone,
                    Krb5kdcStatistic::FinishProcessAsReqNameExp,
                    Krb5kdcStatistic::FinishProcessAsReqServiceExp,
                    Krb5kdcStatistic::FinishProcessAsReqBadPvno,
                    Krb5kdcStatistic::FinishProcessAsReqCOldMastKvno,
                    Krb5kdcStatistic::FinishProcessAsReqSOldMastKvno,
                    Krb5kdcStatistic::FinishProcessAsReqCPrincipalUnknown,
                    Krb5kdcStatistic::FinishProcessAsReqSPrincipalUnknown,
                    Krb5kdcStatistic::FinishProcessAsReqPrincipalNotUnique,
                    Krb5kdcStatistic::FinishProcessAsReqNullKey,
                    Krb5kdcStatistic::FinishProcessAsReqCannotPostdate,
                    Krb5kdcStatistic::FinishProcessAsReqNeverValid,
                    Krb5kdcStatistic::FinishProcessAsReqPolicy,
                    Krb5kdcStatistic::FinishProcessAsReqBadoption,
                    Krb5kdcStatistic::FinishProcessAsReqEtypeNosupp,
                    Krb5kdcStatistic::FinishProcessAsReqSumtypeNosupp,
                    Krb5kdcStatistic::FinishProcessAsReqPadataTypeNosupp,
                    Krb5kdcStatistic::FinishProcessAsReqTrtypeNosupp,
                    Krb5kdcStatistic::FinishProcessAsReqClientRevoked,
                    Krb5kdcStatistic::FinishProcessAsReqServiceRevoked,
                    Krb5kdcStatistic::FinishProcessAsReqTgtRevoked,
                    Krb5kdcStatistic::FinishProcessAsReqClientNotyet,
                    Krb5kdcStatistic::FinishProcessAsReqServiceNotyet,
                    Krb5kdcStatistic::FinishProcessAsReqKeyExp,
                    Krb5kdcStatistic::FinishProcessAsReqPreauthFailed,
                    Krb5kdcStatistic::FinishProcessAsReqPreauthRequired,
                    Krb5kdcStatistic::FinishProcessAsReqServerNomatch,
                    Krb5kdcStatistic::FinishProcessAsReqMustUseUser2user,
                    Krb5kdcStatistic::FinishProcessAsReqPathNotAccepted,
                    Krb5kdcStatistic::FinishProcessAsReqSvcUnavailable,
                ]
                .to_vec(),
            );

            probes.add_user_probe(
                String::from("finish_dispatch_cache"),
                String::from("count_finish_dispatch_cache"),
                ProbeLocation::Entry,
                self.path.clone(),
                [
                    Krb5kdcStatistic::FinishDispatchCacheUnknown,
                    Krb5kdcStatistic::FinishDispatchCacheNone,
                    Krb5kdcStatistic::FinishDispatchCacheNameExp,
                    Krb5kdcStatistic::FinishDispatchCacheServiceExp,
                    Krb5kdcStatistic::FinishDispatchCacheBadPvno,
                    Krb5kdcStatistic::FinishDispatchCacheCOldMastKvno,
                    Krb5kdcStatistic::FinishDispatchCacheSOldMastKvno,
                    Krb5kdcStatistic::FinishDispatchCacheCPrincipalUnknown,
                    Krb5kdcStatistic::FinishDispatchCacheSPrincipalUnknown,
                    Krb5kdcStatistic::FinishDispatchCachePrincipalNotUnique,
                    Krb5kdcStatistic::FinishDispatchCacheNullKey,
                    Krb5kdcStatistic::FinishDispatchCacheCannotPostdate,
                    Krb5kdcStatistic::FinishDispatchCacheNeverValid,
                    Krb5kdcStatistic::FinishDispatchCachePolicy,
                    Krb5kdcStatistic::FinishDispatchCacheBadoption,
                    Krb5kdcStatistic::FinishDispatchCacheEtypeNosupp,
                    Krb5kdcStatistic::FinishDispatchCacheSumtypeNosupp,
                    Krb5kdcStatistic::FinishDispatchCachePadataTypeNosupp,
                    Krb5kdcStatistic::FinishDispatchCacheTrtypeNosupp,
                    Krb5kdcStatistic::FinishDispatchCacheClientRevoked,
                    Krb5kdcStatistic::FinishDispatchCacheServiceRevoked,
                    Krb5kdcStatistic::FinishDispatchCacheTgtRevoked,
                    Krb5kdcStatistic::FinishDispatchCacheClientNotyet,
                    Krb5kdcStatistic::FinishDispatchCacheServiceNotyet,
                    Krb5kdcStatistic::FinishDispatchCacheKeyExp,
                    Krb5kdcStatistic::FinishDispatchCachePreauthFailed,
                    Krb5kdcStatistic::FinishDispatchCachePreauthRequired,
                    Krb5kdcStatistic::FinishDispatchCacheServerNomatch,
                    Krb5kdcStatistic::FinishDispatchCacheMustUseUser2user,
                    Krb5kdcStatistic::FinishDispatchCachePathNotAccepted,
                    Krb5kdcStatistic::FinishDispatchCacheSvcUnavailable,
                ]
                .to_vec(),
            );

            probes.add_user_probe(
                String::from("process_tgs_req"),
                String::from("count_process_tgs_req"),
                ProbeLocation::Return,
                self.path.clone(),
                [
                    Krb5kdcStatistic::ProcessTgsReqUnknown,
                    Krb5kdcStatistic::ProcessTgsReqNone,
                    Krb5kdcStatistic::ProcessTgsReqNameExp,
                    Krb5kdcStatistic::ProcessTgsReqServiceExp,
                    Krb5kdcStatistic::ProcessTgsReqBadPvno,
                    Krb5kdcStatistic::ProcessTgsReqCOldMastKvno,
                    Krb5kdcStatistic::ProcessTgsReqSOldMastKvno,
                    Krb5kdcStatistic::ProcessTgsReqCPrincipalUnknown,
                    Krb5kdcStatistic::ProcessTgsReqSPrincipalUnknown,
                    Krb5kdcStatistic::ProcessTgsReqPrincipalNotUnique,
                    Krb5kdcStatistic::ProcessTgsReqNullKey,
                    Krb5kdcStatistic::ProcessTgsReqCannotPostdate,
                    Krb5kdcStatistic::ProcessTgsReqNeverValid,
                    Krb5kdcStatistic::ProcessTgsReqPolicy,
                    Krb5kdcStatistic::ProcessTgsReqBadoption,
                    Krb5kdcStatistic::ProcessTgsReqEtypeNosupp,
                    Krb5kdcStatistic::ProcessTgsReqSumtypeNosupp,
                    Krb5kdcStatistic::ProcessTgsReqPadataTypeNosupp,
                    Krb5kdcStatistic::ProcessTgsReqTrtypeNosupp,
                    Krb5kdcStatistic::ProcessTgsReqClientRevoked,
                    Krb5kdcStatistic::ProcessTgsReqServiceRevoked,
                    Krb5kdcStatistic::ProcessTgsReqTgtRevoked,
                    Krb5kdcStatistic::ProcessTgsReqClientNotyet,
                    Krb5kdcStatistic::ProcessTgsReqServiceNotyet,
                    Krb5kdcStatistic::ProcessTgsReqKeyExp,
                    Krb5kdcStatistic::ProcessTgsReqPreauthFailed,
                    Krb5kdcStatistic::ProcessTgsReqPreauthRequired,
                    Krb5kdcStatistic::ProcessTgsReqServerNomatch,
                    Krb5kdcStatistic::ProcessTgsReqMustUseUser2user,
                    Krb5kdcStatistic::ProcessTgsReqPathNotAccepted,
                    Krb5kdcStatistic::ProcessTgsReqSvcUnavailable,
                ]
                .to_vec(),
            );

            // load + attach the user probes that are required to the bpf instance.
            probes.try_attach_to_bpf(
                &mut bpf,
                self.statistics.as_slice(),
                Some(self.common.config().fault_tolerant()),
            )?;

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
