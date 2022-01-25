// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use nvml_wrapper::NVML;
use serde_derive::Deserialize;
use strum::IntoEnumIterator;

use crate::config::SamplerConfig;

use super::stat::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NvidiaConfig {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    interval: Option<usize>,
    #[serde(default = "crate::common::default_percentiles")]
    percentiles: Vec<f64>,
    #[serde(default = "default_statistics")]
    pub(crate) statistics: Vec<NvidiaConfigStatistic>,
}

impl Default for NvidiaConfig {
    fn default() -> Self {
        Self {
            enabled: Default::default(),
            interval: Default::default(),
            percentiles: crate::common::default_percentiles(),
            statistics: default_statistics(),
        }
    }
}

fn default_statistics() -> Vec<NvidiaConfigStatistic> {
    NvidiaConfigStatistic::iter().collect()
}

impl SamplerConfig for NvidiaConfig {
    type Statistic = NvidiaStatistic;

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn interval(&self) -> Option<usize> {
        self.interval
    }

    fn percentiles(&self) -> &[f64] {
        &self.percentiles
    }

    fn statistics(&self) -> Vec<<Self as SamplerConfig>::Statistic> {
        let mut enabled = Vec::new();
        if let Ok(nvml) = NVML::builder().init() {
            let devices = nvml.device_count().unwrap_or(0);
            for statistic in self.statistics.iter() {
                for id in 0..devices {
                    match statistic {
                        NvidiaConfigStatistic::GpuTemperature => {
                            enabled.push(NvidiaStatistic::GpuTemperature(id));
                        }
                        NvidiaConfigStatistic::GpuUtilization => {
                            enabled.push(NvidiaStatistic::GpuUtilization(id));
                        }
                        NvidiaConfigStatistic::MemoryEccEnabled => {
                            enabled.push(NvidiaStatistic::MemoryEccEnabled(id));
                        }
                        NvidiaConfigStatistic::MemoryEccSbe => {
                            enabled.push(NvidiaStatistic::MemoryEccSbe(id));
                        }
                        NvidiaConfigStatistic::MemoryEccDbe => {
                            enabled.push(NvidiaStatistic::MemoryEccDbe(id));
                        }
                        NvidiaConfigStatistic::MemoryUtilization => {
                            enabled.push(NvidiaStatistic::MemoryUtilization(id));
                        }
                        NvidiaConfigStatistic::EncoderUtilization => {
                            enabled.push(NvidiaStatistic::EncoderUtilization(id));
                        }
                        NvidiaConfigStatistic::DecoderUtilization => {
                            enabled.push(NvidiaStatistic::DecoderUtilization(id));
                        }
                        NvidiaConfigStatistic::PowerUsage => {
                            enabled.push(NvidiaStatistic::PowerUsage(id));
                        }
                        NvidiaConfigStatistic::PowerLimit => {
                            enabled.push(NvidiaStatistic::PowerLimit(id));
                        }
                        NvidiaConfigStatistic::EnergyConsumption => {
                            enabled.push(NvidiaStatistic::EnergyConsumption(id));
                        }
                        NvidiaConfigStatistic::ClockSMCurrent => {
                            enabled.push(NvidiaStatistic::ClockSMCurrent(id));
                        }
                        NvidiaConfigStatistic::ClockMemoryCurrent => {
                            enabled.push(NvidiaStatistic::ClockMemoryCurrent(id));
                        }
                        NvidiaConfigStatistic::PcieReplay => {
                            enabled.push(NvidiaStatistic::PcieReplay(id));
                        }
                        NvidiaConfigStatistic::PcieRxThroughput => {
                            enabled.push(NvidiaStatistic::PcieRxThroughput(id));
                        }
                        NvidiaConfigStatistic::PcieTxThroughput => {
                            enabled.push(NvidiaStatistic::PcieTxThroughput(id));
                        }
                        NvidiaConfigStatistic::MemoryFbFree => {
                            enabled.push(NvidiaStatistic::MemoryFbFree(id));
                        }
                        NvidiaConfigStatistic::MemoryFbTotal => {
                            enabled.push(NvidiaStatistic::MemoryFbTotal(id));
                        }
                        NvidiaConfigStatistic::MemoryFbUsed => {
                            enabled.push(NvidiaStatistic::MemoryFbUsed(id));
                        }
                        NvidiaConfigStatistic::MemoryRetiredSbe => {
                            enabled.push(NvidiaStatistic::MemoryRetiredSbe(id));
                        }
                        NvidiaConfigStatistic::MemoryRetiredDbe => {
                            enabled.push(NvidiaStatistic::MemoryRetiredDbe(id));
                        }
                        NvidiaConfigStatistic::MemoryRetiredPending => {
                            enabled.push(NvidiaStatistic::MemoryRetiredPending(id));
                        }
                        NvidiaConfigStatistic::ProcessesCompute => {
                            enabled.push(NvidiaStatistic::ProcessesCompute(id));
                        }
                    }
                }
            }
        }
        enabled
    }
}
