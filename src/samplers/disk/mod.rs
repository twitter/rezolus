// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use std::time::*;

use async_trait::async_trait;
use regex::Regex;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::common::bpf::*;
use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::Sampler;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Disk {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
}

#[async_trait]
impl Sampler for Disk {
    type Statistic = DiskStatistic;

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
            fatal!("failed to initialize disk sampler");
        } else {
            error!("failed to initialize disk sampler");
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().disk()
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

        self.map_result(self.sample_diskstats().await)?;
        #[cfg(feature = "bpf")]
        self.map_result(self.sample_bpf())?;

        Ok(())
    }

    // fn summary(&self, statistic: &Self::Statistic) -> Option<Summary> {
    //     let precision = if statistic.bpf_table().is_some() {
    //         2
    //     } else {
    //         3
    //     };

    //     let max = if statistic.bpf_table().is_some() {
    //         SECOND
    //     } else {
    //         TEBIBYTE
    //     };

    //     Some(Summary::histogram(
    //         max,
    //         precision,
    //         Some(self.general_config().window()),
    //     ))
    // }
}

impl Disk {
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
                let code = include_str!("bpf.c");
                let mut bpf = bcc::BPF::new(code)?;
                // load + attach kprobes!
                bcc::Kprobe::new()
                    .handler("trace_pid_start")
                    .function("blk_account_io_start")
                    .attach(&mut bpf)?;
                bcc::Kprobe::new()
                    .handler("trace_req_start")
                    .function("blk_start_request")
                    .attach(&mut bpf)?;
                bcc::Kprobe::new()
                    .handler("trace_req_start")
                    .function("blk_mq_start_request")
                    .attach(&mut bpf)?;
                bcc::Kprobe::new()
                    .handler("do_count")
                    .function("blk_account_io_completion")
                    .attach(&mut bpf)?;

                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
            }
        }

        Ok(())
    }

    async fn sample_diskstats(&self) -> Result<(), std::io::Error> {
        // process diskstats
        let file = File::open("/proc/diskstats").await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut result = HashMap::new();
        // regex to match devices we care about
        let re = Regex::new(r"^((sd[a-z]+)|(hd[a-z]+)|(nvme\d+n\d+))$")
            .expect("failed to compile regex");
        while let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if re.is_match(parts.get(2).unwrap_or(&"unknown")) {
                for statistic in self.sampler_config().statistics() {
                    if let Some(field) = statistic.diskstat_field() {
                        if !result.contains_key(&statistic) {
                            result.insert(statistic, 0);
                        }
                        let current = result.get_mut(&statistic).unwrap();
                        *current += parts
                            .get(field)
                            .map(|v| v.parse().unwrap_or(0))
                            .unwrap_or(0);
                    }
                }
            }
        }

        let time = Instant::now();
        for (stat, value) in result {
            let value = match stat {
                DiskStatistic::BandwidthWrite
                | DiskStatistic::BandwidthRead
                | DiskStatistic::BandwidthDiscard => value * 512,
                _ => value,
            };
            let _ = self.metrics().record_counter(&stat, time, value);
        }
        Ok(())
    }

    #[cfg(feature = "bpf")]
    fn sample_bpf(&self) -> Result<(), std::io::Error> {
        if self.bpf_last.lock().unwrap().elapsed()
            >= Duration::new(self.general_config().window().try_into().unwrap(), 0)
        {
            let time = Instant::now();
            if let Some(ref bpf) = self.bpf {
                let bpf = bpf.lock().unwrap();
                for statistic in self.sampler_config().statistics() {
                    if let Some(table) = statistic.bpf_table() {
                        let mut table = (*bpf).inner.table(table);

                        for (&value, &count) in &map_from_table(&mut table) {
                            if count > 0 {
                                let _ = self.metrics().record_bucket(
                                    &statistic,
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
