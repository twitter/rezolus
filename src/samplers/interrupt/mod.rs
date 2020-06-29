// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;

use async_trait::async_trait;
use rustcommon_metrics::*;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::common::*;
use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::Sampler;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Interrupt {
    common: Common,
}

#[async_trait]
impl Sampler for Interrupt {
    type Statistic = InterruptStatistic;

    fn new(common: Common) -> Result<Self, failure::Error> {
        Ok(Self { common })
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

        Ok(())
    }

    fn summary(&self, _statistic: &Self::Statistic) -> Option<Summary> {
        let max = crate::common::hardware_threads().unwrap_or(1024) * SECOND;
        Some(Summary::histogram(
            max,
            3,
            Some(self.general_config().window()),
        ))
    }
}

impl Interrupt {
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

        let time = time::precise_time_ns();
        for stat in self.sampler_config().statistics() {
            if let Some(value) = result.get(stat) {
                self.metrics().record_counter(stat, time, *value);
            }
        }

        Ok(())
    }
}
