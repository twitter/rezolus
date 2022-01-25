// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "bpf")]
use std::collections::HashSet;

use std::sync::{Arc, Mutex};
use tokio::fs::File;

use async_trait::async_trait;

use crate::common::bpf::*;
use crate::config::SamplerConfig;
use crate::samplers::{Common, Sampler};
use crate::*;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Tcp {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
    proc_net_snmp: Option<File>,
    proc_net_netstat: Option<File>,
    statistics: Vec<TcpStatistic>,
}

#[async_trait]
impl Sampler for Tcp {
    type Statistic = TcpStatistic;
    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let fault_tolerant = common.config.general().fault_tolerant();
        let statistics = common.config().samplers().tcp().statistics();

        #[allow(unused_mut)]
        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
            proc_net_snmp: None,
            proc_net_netstat: None,
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
        if common.config().samplers().tcp().enabled() {
            if let Ok(mut sampler) = Self::new(common.clone()) {
                common.runtime().spawn(async move {
                    loop {
                        let _ = sampler.sample().await;
                    }
                });
            } else if !common.config.fault_tolerant() {
                fatal!("failed to initialize tcp sampler");
            } else {
                error!("failed to initialize tcp sampler");
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
        self.common.config().samplers().tcp()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }

        debug!("sampling");

        let r = self.sample_snmp().await;
        self.map_result(r)?;

        let r = self.sample_netstat().await;
        self.map_result(r)?;

        // sample bpf
        #[cfg(feature = "bpf")]
        self.map_result(self.sample_bpf())?;

        Ok(())
    }
}

impl Tcp {
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

    fn initialize_bpf(&mut self) -> Result<(), anyhow::Error> {
        #[cfg(feature = "bpf")]
        {
            if self.enabled() && self.bpf_enabled() {
                debug!("initializing bpf");
                // load the code and compile
                let code = include_str!("bpf.c");
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

                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })))
            }
        }

        Ok(())
    }

    async fn sample_snmp(&mut self) -> Result<(), std::io::Error> {
        if self.proc_net_snmp.is_none() {
            let file = File::open("/proc/net/snmp").await?;
            self.proc_net_snmp = Some(file);
        }
        if let Some(file) = &mut self.proc_net_snmp {
            let parsed = crate::common::nested_map_from_file(file).await?;
            let time = Instant::now();
            for statistic in &self.statistics {
                if let Some((pkey, lkey)) = statistic.keys() {
                    if let Some(inner) = parsed.get(pkey) {
                        if let Some(value) = inner.get(lkey) {
                            let _ = self.metrics().record_counter(statistic, time, *value);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn sample_netstat(&mut self) -> Result<(), std::io::Error> {
        if self.proc_net_netstat.is_none() {
            let file = File::open("/proc/net/netstat").await?;
            self.proc_net_netstat = Some(file);
        }
        if let Some(file) = &mut self.proc_net_netstat {
            let parsed = crate::common::nested_map_from_file(file).await?;
            let time = Instant::now();
            for statistic in &self.statistics {
                if let Some((pkey, lkey)) = statistic.keys() {
                    if let Some(inner) = parsed.get(pkey) {
                        if let Some(value) = inner.get(lkey) {
                            let _ = self.metrics().record_counter(statistic, time, *value);
                        }
                    }
                }
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
                    // if statistic is Counter
                    match statistic.source() {
                        Source::Counter => {
                            if let Ok(table) = &(*bpf).inner.table(statistic.bpf_table().unwrap()) {
                                let count = crate::common::bpf::parse_u64(
                                    table.iter().next().unwrap().value,
                                );
                                let _ = self.metrics().record_counter(statistic, time, count);
                            }
                        }
                        // if it's distribution
                        Source::Distribution => {
                            if let Ok(mut table) =
                                (*bpf).inner.table(statistic.bpf_table().unwrap())
                            {
                                for (&value, &count) in &map_from_table(&mut table) {
                                    if count > 0 {
                                        let _ = self.metrics().record_bucket(
                                            statistic,
                                            time,
                                            // in bpf everything is in micro, we convert it back to nano for alignment.
                                            value * 1000,
                                            count,
                                        );
                                    }
                                }
                            }
                        }
                        _ => (), // we do not support other types
                    }
                }
            }
            *self.bpf_last.lock().unwrap() = Instant::now();
        }
        Ok(())
    }
}
