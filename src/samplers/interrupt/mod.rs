// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;

use async_trait::async_trait;
use metrics::*;
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
            for i in 1..cores.unwrap() {
                sum += parts.get(i).unwrap_or(&"0").parse().unwrap_or(0);
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
                    Some(label) => {
                        if label.starts_with("mlx") || label.starts_with("eth") {
                            InterruptStatistic::Network
                        } else if label.starts_with("nvme") || label.starts_with("vmd") {
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
            result.insert(stat, sum);
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
