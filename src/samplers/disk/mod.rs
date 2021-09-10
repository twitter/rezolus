// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::io::SeekFrom;
use std::sync::{Arc, Mutex};
use std::time::*;

use async_trait::async_trait;
use regex::Regex;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};

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
    proc_diskstats: Option<File>,
    disk_regex: Option<Regex>,
    statistics: Vec<DiskStatistic>,
    stats: DiskStats,
}

#[async_trait]
impl Sampler for Disk {
    type Statistic = DiskStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let fault_tolerant = common.config.general().fault_tolerant();
        let statistics = common.config().samplers().disk().statistics();

        #[allow(unused_mut)]
        let mut sampler = Self {
            stats: DiskStats::new(
                crate::samplers::static_samples(
                    common.config().samplers().disk(),
                    common.config().general(),
                ),
                Duration::from_secs(common.config().general().window() as _),
                common.config().samplers().disk().percentiles(),
            ),
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
            proc_diskstats: None,
            disk_regex: None,
            statistics,
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
        if common.config().samplers().disk().enabled() {
            if let Ok(mut sampler) = Self::new(common.clone()) {
                common.runtime().spawn(async move {
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

        let r = self.sample_diskstats().await;
        self.map_result(r)?;
        #[cfg(feature = "bpf")]
        self.map_result(self.sample_bpf())?;

        Ok(())
    }
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

    fn initialize_bpf(&mut self) -> Result<(), anyhow::Error> {
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
                if let Ok(results) = bpf.get_kprobe_functions("blk_start_request") {
                    if !results.is_empty() {
                        bcc::Kprobe::new()
                            .handler("trace_req_start")
                            .function("blk_start_request")
                            .attach(&mut bpf)?;
                    }
                }
                bcc::Kprobe::new()
                    .handler("trace_req_start")
                    .function("blk_mq_start_request")
                    .attach(&mut bpf)?;
                if let Ok(results) = bpf.get_kprobe_functions("blk_account_io_completion") {
                    if !results.is_empty() {
                        bcc::Kprobe::new()
                            .handler("do_count")
                            .function("blk_account_io_completion")
                            .attach(&mut bpf)?;
                    } else {
                        bcc::Kprobe::new()
                            .handler("do_count")
                            .function("blk_account_io_done")
                            .attach(&mut bpf)?;
                    }
                }
                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
            }
        }

        Ok(())
    }

    async fn sample_diskstats(&mut self) -> Result<(), std::io::Error> {
        if self.proc_diskstats.is_none() {
            let file = File::open("/proc/diskstats").await?;
            self.proc_diskstats = Some(file);
        }

        if self.disk_regex.is_none() {
            let re = Regex::new(r"^((sd[a-z]+)|(hd[a-z]+)|(nvme\d+n\d+))$")
                .expect("failed to compile regex");
            self.disk_regex = Some(re);
        }

        if let Some(file) = &mut self.proc_diskstats {
            let mut operations_read = None;
            let mut operations_write = None;
            let mut operations_discard = None;
            let mut bandwidth_read = None;
            let mut bandwidth_write = None;
            let mut bandwidth_discard = None;

            file.seek(SeekFrom::Start(0)).await?;
            if let Some(re) = &mut self.disk_regex {
                let mut reader = BufReader::new(file);
                let mut line = String::new();
                while reader.read_line(&mut line).await? > 0 {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if re.is_match(parts.get(2).unwrap_or(&"unknown")) {
                        for (id, part) in parts.iter().enumerate() {
                            match id {
                                3 => *operations_read.get_or_insert(0) += part.parse().unwrap_or(0),
                                5 => *bandwidth_read.get_or_insert(0) += part.parse().unwrap_or(0),
                                7 => {
                                    *operations_write.get_or_insert(0) += part.parse().unwrap_or(0)
                                }
                                9 => *bandwidth_write.get_or_insert(0) += part.parse().unwrap_or(0),
                                14 => {
                                    *operations_discard.get_or_insert(0) +=
                                        part.parse().unwrap_or(0)
                                }
                                16 => {
                                    *bandwidth_discard.get_or_insert(0) += part.parse().unwrap_or(0)
                                }
                                _ => (),
                            }
                        }
                    }
                    line.clear();
                }
                let time = Instant::now();
                if_block! {
                    if let Some(value) = operations_read => self.stats.operations_read.store(time, value);
                    if let Some(value) = operations_write => self.stats.operations_write.store(time, value);
                    if let Some(value) = operations_discard => self.stats.operations_discard.store(time, value);
                    if let Some(value) = bandwidth_read => self.stats.bandwidth_read.store(time, value * 512);
                    if let Some(value) = bandwidth_write => self.stats.bandwidth_write.store(time, value * 512);
                    if let Some(value) = bandwidth_discard => self.stats.bandwidth_discard.store(time, value * 512);
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "bpf")]
    fn sample_bpf(&self) -> Result<(), std::io::Error> {
        if self.bpf_last.lock().unwrap().elapsed()
            >= Duration::from_secs(self.general_config().window() as _)
        {
            let time = Instant::now();
            if let Some(ref bpf) = self.bpf {
                let bpf = bpf.lock().unwrap();

                macro_rules! record_bucket {
                    ($metric:expr, $stat:expr, $time:expr) => {{
                        let ref metric = $metric;
                        let time = $time;

                        if let Ok(mut table) = bpf.inner.table($stat.bpf_table().unwrap()) {
                            for (&value, &count) in &map_from_table(&mut table) {
                                if count == 0 {
                                    continue;
                                }
                                let _ = metric.insert(time, value * crate::MICROSECOND, count);
                            }
                        }
                    }};
                }

                record_bucket!(self.stats.latency_read, DiskStatistic::LatencyRead, time);
                record_bucket!(self.stats.latency_write, DiskStatistic::LatencyWrite, time);
                record_bucket!(
                    self.stats.device_latency_read,
                    DiskStatistic::DeviceLatencyRead,
                    time
                );
                record_bucket!(
                    self.stats.device_latency_write,
                    DiskStatistic::DeviceLatencyWrite,
                    time
                );
                record_bucket!(
                    self.stats.queue_latency_read,
                    DiskStatistic::QueueLatencyRead,
                    time
                );
                record_bucket!(
                    self.stats.queue_latency_write,
                    DiskStatistic::QueueLatencyWrite,
                    time
                );
                record_bucket!(self.stats.io_size_read, DiskStatistic::IoSizeRead, time);
                record_bucket!(self.stats.io_size_write, DiskStatistic::IoSizeWrite, time);
            }
            *self.bpf_last.lock().unwrap() = Instant::now();
        }
        Ok(())
    }
}
