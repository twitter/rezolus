// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
#[cfg(feature = "bpf")]
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::*;
use tokio::io::SeekFrom;

use async_trait::async_trait;
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
pub struct Interrupt {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
    proc_interrupts: Option<File>,
    statistics: Vec<InterruptStatistic>,
}

#[async_trait]
impl Sampler for Interrupt {
    type Statistic = InterruptStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let fault_tolerant = common.config.general().fault_tolerant();
        let statistics = common.config().samplers().interrupt().statistics();

        #[allow(unused_mut)]
        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
            proc_interrupts: None,
            statistics,
        };

        if let Err(e) = sampler.initialize_bpf() {
            error!("failed to initializing bpf: {}", e);
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
        if common.config().samplers().interrupt().enabled() {
            if let Ok(mut interrupt) = Interrupt::new(common.clone()) {
                common.runtime().spawn(async move {
                    loop {
                        let _ = interrupt.sample().await;
                    }
                });
            } else if !common.config.fault_tolerant() {
                fatal!("failed to initialize interrupt sampler");
            } else {
                error!("failed to initialize interrupt sampler");
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
        self.common.config().samplers().interrupt()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }

        debug!("sampling");

        self.sample_interrupt().await?;

        #[cfg(feature = "bpf")]
        self.map_result(self.sample_bpf())?;

        Ok(())
    }
}

impl Interrupt {
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

    async fn sample_interrupt(&mut self) -> Result<(), std::io::Error> {
        if self.proc_interrupts.is_none() {
            let file = File::open("/proc/interrupts").await?;
            self.proc_interrupts = Some(file);
        }

        let mut result = HashMap::<InterruptStatistic, u64>::new();
        let mut cores: Option<usize> = None;

        if let Some(file) = &mut self.proc_interrupts {
            file.seek(SeekFrom::Start(0)).await?;
            let mut reader = BufReader::new(file);
            let mut line = String::new();

            loop {
                line.clear();

                if reader.read_line(&mut line).await? == 0 {
                    break;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if cores.is_none() {
                    cores = Some(parts.len());
                    continue;
                }
                let mut sum = 0;
                let mut node0 = 0;
                let mut node1 = 0;
                let cores = cores.unwrap();

                for i in 0..cores {
                    let count = parts.get(i + 1).unwrap_or(&"0").parse().unwrap_or(0);
                    sum += count;

                    let node = self.common.hardware_info().get_numa(i as u64).unwrap_or(0);
                    match node {
                        0 => node0 += count,
                        1 => node1 += count,
                        _ => {}
                    }
                }
                let stat = match parts.get(0) {
                    Some(&"NMI:") => InterruptStatistic::NonMaskable,
                    Some(&"LOC:") => InterruptStatistic::LocalTimer,
                    Some(&"SPU:") => InterruptStatistic::Spurious,
                    Some(&"PMI:") => InterruptStatistic::PerformanceMonitoring,
                    Some(&"RES:") => InterruptStatistic::Rescheduling,
                    Some(&"TLB:") => InterruptStatistic::TlbShootdowns,
                    Some(&"TRM:") => InterruptStatistic::ThermalEvent,
                    Some(&"MCE:") => InterruptStatistic::MachineCheckException,
                    _ => match parts.last() {
                        Some(&"timer") => InterruptStatistic::Timer,
                        Some(&"rtc0") => InterruptStatistic::RealTimeClock,
                        Some(&"vmd") => {
                            if let Some(previous) = result.get_mut(&InterruptStatistic::Node0Nvme) {
                                *previous += node0;
                            } else {
                                result.insert(InterruptStatistic::Node0Nvme, node0);
                            }
                            if let Some(previous) = result.get_mut(&InterruptStatistic::Node1Nvme) {
                                *previous += node1;
                            } else {
                                result.insert(InterruptStatistic::Node1Nvme, node1);
                            }
                            InterruptStatistic::Nvme
                        }
                        Some(label) => {
                            if label.starts_with("mlx")
                                || label.starts_with("eth")
                                || label.starts_with("enp")
                            {
                                if let Some(previous) =
                                    result.get_mut(&InterruptStatistic::Node0Network)
                                {
                                    *previous += node0;
                                } else {
                                    result.insert(InterruptStatistic::Node0Network, node0);
                                }
                                if let Some(previous) =
                                    result.get_mut(&InterruptStatistic::Node1Network)
                                {
                                    *previous += node1;
                                } else {
                                    result.insert(InterruptStatistic::Node1Network, node1);
                                }
                                InterruptStatistic::Network
                            } else if label.starts_with("nvme") {
                                if let Some(previous) =
                                    result.get_mut(&InterruptStatistic::Node0Nvme)
                                {
                                    *previous += node0;
                                } else {
                                    result.insert(InterruptStatistic::Node0Nvme, node0);
                                }
                                if let Some(previous) =
                                    result.get_mut(&InterruptStatistic::Node1Nvme)
                                {
                                    *previous += node1;
                                } else {
                                    result.insert(InterruptStatistic::Node1Nvme, node1);
                                }
                                InterruptStatistic::Nvme
                            } else {
                                continue;
                            }
                        }
                        None => {
                            continue;
                        }
                    },
                };
                if let Some(previous) = result.get_mut(&stat) {
                    *previous += sum;
                } else {
                    result.insert(stat, sum);
                }
                if let Some(previous) = result.get_mut(&InterruptStatistic::Total) {
                    *previous += sum;
                } else {
                    result.insert(InterruptStatistic::Total, sum);
                }
                if let Some(previous) = result.get_mut(&InterruptStatistic::Node0Total) {
                    *previous += node0;
                } else {
                    result.insert(InterruptStatistic::Node0Total, node0);
                }
                if let Some(previous) = result.get_mut(&InterruptStatistic::Node1Total) {
                    *previous += node1;
                } else {
                    result.insert(InterruptStatistic::Node1Total, node1);
                }
            }
        }

        let time = Instant::now();
        for stat in &self.statistics {
            if let Some(value) = result.get(stat) {
                let _ = self.metrics().record_counter(stat, time, *value);
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
