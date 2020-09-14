// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::*;

use async_trait::async_trait;
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
pub struct Interrupt {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
}

#[async_trait]
impl Sampler for Interrupt {
    type Statistic = InterruptStatistic;

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
        if let Ok(mut interrupt) = Interrupt::new(common.clone()) {
            common.handle.spawn(async move {
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
        self.register();

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

                let code = include_str!("bpf.c");
                let mut bpf = bcc::BPF::new(code)?;

                bcc::Kprobe::new()
                    .handler("hardirq_entry")
                    .function("handle_irq_event_percpu")
                    .attach(&mut bpf)?;
                bcc::Kretprobe::new()
                    .handler("hardirq_exit")
                    .function("handle_irq_event_percpu")
                    .attach(&mut bpf)?;
                bcc::Tracepoint::new()
                    .handler("softirq_entry")
                    .subsystem("irq")
                    .tracepoint("softirq_entry")
                    .attach(&mut bpf)?;
                bcc::Tracepoint::new()
                    .handler("softirq_exit")
                    .subsystem("irq")
                    .tracepoint("softirq_exit")
                    .attach(&mut bpf)?;

                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })))
            }
        }

        Ok(())
    }

    async fn sample_interrupt(&self) -> Result<(), std::io::Error> {
        let file = File::open("/proc/interrupts").await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut result = HashMap::new();

        let mut cores: Option<usize> = None;
        while let Some(line) = lines.next_line().await? {
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
                // Assumes the system is split into 2 NUMA nodes with
                // hyperthreading enabled and that cores are arranged as follows
                if i < (cores / 4) || (i >= (cores / 2) && i < (3 * cores / 4)) {
                    node0 += count;
                } else {
                    node1 += count;
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
                            result.insert(InterruptStatistic::Node0Nvme, sum);
                        }
                        if let Some(previous) = result.get_mut(&InterruptStatistic::Node1Nvme) {
                            *previous += node1;
                        } else {
                            result.insert(InterruptStatistic::Node1Nvme, sum);
                        }
                        InterruptStatistic::Nvme
                    }
                    Some(label) => {
                        if label.starts_with("mlx") || label.starts_with("eth") {
                            if let Some(previous) =
                                result.get_mut(&InterruptStatistic::Node0Network)
                            {
                                *previous += node0;
                            } else {
                                result.insert(InterruptStatistic::Node0Network, sum);
                            }
                            if let Some(previous) =
                                result.get_mut(&InterruptStatistic::Node1Network)
                            {
                                *previous += node1;
                            } else {
                                result.insert(InterruptStatistic::Node1Network, sum);
                            }
                            InterruptStatistic::Network
                        } else if label.starts_with("nvme") {
                            if let Some(previous) = result.get_mut(&InterruptStatistic::Node0Nvme) {
                                *previous += node0;
                            } else {
                                result.insert(InterruptStatistic::Node0Nvme, sum);
                            }
                            if let Some(previous) = result.get_mut(&InterruptStatistic::Node1Nvme) {
                                *previous += node1;
                            } else {
                                result.insert(InterruptStatistic::Node1Nvme, sum);
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

        let time = Instant::now();
        for stat in self.sampler_config().statistics() {
            if let Some(value) = result.get(&stat) {
                let _ = self.metrics().record_counter(&stat, time, *value);
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
