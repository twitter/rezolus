// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::convert::TryInto;

use async_trait::async_trait;
use nvml_wrapper::enum_wrappers::device::*;
use nvml_wrapper::NVML;

use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::*;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Nvidia {
    common: Common,
    nvml: NVML,
    statistics: Vec<NvidiaStatistic>,
}

#[async_trait]
impl Sampler for Nvidia {
    type Statistic = NvidiaStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let statistics = common.config().samplers().nvidia().statistics();
        match NVML::builder().init() {
            Ok(nvml) => {
                #[allow(unused_mut)]
                let mut sampler = Self {
                    common,
                    nvml,
                    statistics,
                };

                if sampler.sampler_config().enabled() {
                    sampler.register();
                }

                Ok(sampler)
            }
            Err(e) => Err(anyhow!("failed to initialize NVML: {}", e)),
        }
    }

    fn spawn(common: Common) {
        debug!("spawning");
        if common.config().samplers().nvidia().enabled() {
            debug!("sampler is enabled");
            if let Ok(mut sampler) = Nvidia::new(common.clone()) {
                common.runtime().spawn(async move {
                    loop {
                        let _ = sampler.sample().await;
                    }
                });
            } else if !common.config.fault_tolerant() {
                fatal!("failed to initialize nvidia sampler");
            } else {
                error!("failed to initialize nvidia sampler");
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
        self.common.config().samplers().nvidia()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }

        debug!("sampling");

        let r = self.sample_nvml().await;
        self.map_result(r)?;

        Ok(())
    }
}

impl Nvidia {
    async fn sample_nvml(&mut self) -> Result<(), std::io::Error> {
        let time = Instant::now();
        let devices = self.nvml.device_count().unwrap_or(0);
        let statistics = &self.common.config().samplers().nvidia().statistics;
        for id in 0..devices {
            if let Ok(device) = self.nvml.device_by_index(id) {
                for statistic in statistics {
                    match statistic {
                        NvidiaConfigStatistic::GpuTemperature => {
                            if let Ok(value) = device.temperature(TemperatureSensor::Gpu) {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::GpuTemperature(id),
                                    time,
                                    value.into(),
                                );
                            }
                        }
                        NvidiaConfigStatistic::MemoryEccEnabled => {
                            if let Ok(value) = device.is_ecc_enabled().map(|v| {
                                if v.currently_enabled {
                                    1_u32
                                } else {
                                    0_u32
                                }
                            }) {
                                let _ = self.metrics().record_counter(
                                    &NvidiaStatistic::MemoryEccEnabled(id),
                                    time,
                                    value.into(),
                                );
                            }
                        }
                        NvidiaConfigStatistic::MemoryEccSbe => {
                            if let Ok(value) = device
                                .total_ecc_errors(MemoryError::Corrected, EccCounter::Aggregate)
                            {
                                let _ = self.metrics().record_counter(
                                    &NvidiaStatistic::MemoryEccSbe(id),
                                    time,
                                    value,
                                );
                            }
                        }
                        NvidiaConfigStatistic::MemoryEccDbe => {
                            if let Ok(value) = device
                                .total_ecc_errors(MemoryError::Uncorrected, EccCounter::Aggregate)
                            {
                                let _ = self.metrics().record_counter(
                                    &NvidiaStatistic::MemoryEccDbe(id),
                                    time,
                                    value,
                                );
                            }
                        }
                        NvidiaConfigStatistic::PowerUsage => {
                            if let Ok(value) = device.power_usage() {
                                let value = (value as f64 / 1000.0).round() as u64;
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::PowerUsage(id),
                                    time,
                                    value,
                                );
                            }
                        }
                        NvidiaConfigStatistic::PowerLimit => {
                            if let Ok(value) = device.enforced_power_limit() {
                                let value = (value as f64 / 1000.0).round() as u64;
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::PowerLimit(id),
                                    time,
                                    value,
                                );
                            }
                        }
                        NvidiaConfigStatistic::EnergyConsumption => {
                            if let Ok(value) = device.total_energy_consumption() {
                                let value = (value as f64 / 1000.0).round() as u64;
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::EnergyConsumption(id),
                                    time,
                                    value,
                                );
                            }
                        }
                        NvidiaConfigStatistic::ClockSMCurrent => {
                            if let Ok(value) = device.clock(Clock::SM, ClockId::Current) {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::ClockSMCurrent(id),
                                    time,
                                    value.into(),
                                );
                            }
                        }
                        NvidiaConfigStatistic::ClockMemoryCurrent => {
                            if let Ok(value) = device.clock(Clock::Memory, ClockId::Current) {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::ClockMemoryCurrent(id),
                                    time,
                                    value.into(),
                                );
                            }
                        }
                        NvidiaConfigStatistic::PcieReplay => {
                            if let Ok(value) = device.pcie_replay_counter() {
                                let _ = self.metrics().record_counter(
                                    &NvidiaStatistic::PcieReplay(id),
                                    time,
                                    value.into(),
                                );
                            }
                        }
                        NvidiaConfigStatistic::PcieRxThroughput => {
                            if let Ok(value) = device.pcie_throughput(PcieUtilCounter::Receive) {
                                let _ = self.metrics().record_counter(
                                    &NvidiaStatistic::PcieRxThroughput(id),
                                    time,
                                    value.into(),
                                );
                            }
                        }
                        NvidiaConfigStatistic::PcieTxThroughput => {
                            if let Ok(value) = device.pcie_throughput(PcieUtilCounter::Send) {
                                let _ = self.metrics().record_counter(
                                    &NvidiaStatistic::PcieTxThroughput(id),
                                    time,
                                    value.into(),
                                );
                            }
                        }
                        NvidiaConfigStatistic::GpuUtilization => {
                            if let Ok(value) = device.utilization_rates() {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::GpuUtilization(id),
                                    time,
                                    value.gpu.into(),
                                );
                            }
                        }
                        NvidiaConfigStatistic::MemoryUtilization => {
                            if let Ok(value) = device.utilization_rates() {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::MemoryUtilization(id),
                                    time,
                                    value.memory.into(),
                                );
                            }
                        }
                        NvidiaConfigStatistic::DecoderUtilization => {
                            if let Ok(value) = device.decoder_utilization() {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::DecoderUtilization(id),
                                    time,
                                    value.utilization as u64 * 100_u64
                                        / value.sampling_period as u64,
                                );
                            }
                        }
                        NvidiaConfigStatistic::EncoderUtilization => {
                            if let Ok(value) = device.encoder_utilization() {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::EncoderUtilization(id),
                                    time,
                                    value.utilization as u64 * 100_u64
                                        / value.sampling_period as u64,
                                );
                            }
                        }
                        NvidiaConfigStatistic::MemoryFbFree => {
                            if let Ok(value) = device.memory_info() {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::MemoryFbFree(id),
                                    time,
                                    value.free,
                                );
                            }
                        }
                        NvidiaConfigStatistic::MemoryFbTotal => {
                            if let Ok(value) = device.memory_info() {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::MemoryFbTotal(id),
                                    time,
                                    value.total,
                                );
                            }
                        }
                        NvidiaConfigStatistic::MemoryFbUsed => {
                            if let Ok(value) = device.memory_info() {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::MemoryFbUsed(id),
                                    time,
                                    value.used,
                                );
                            }
                        }
                        NvidiaConfigStatistic::MemoryRetiredSbe => {
                            if let Ok(value) =
                                device.retired_pages(RetirementCause::MultipleSingleBitEccErrors)
                            {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::MemoryRetiredSbe(id),
                                    time,
                                    value.len().try_into().unwrap(),
                                );
                            }
                        }
                        NvidiaConfigStatistic::MemoryRetiredDbe => {
                            if let Ok(value) =
                                device.retired_pages(RetirementCause::DoubleBitEccError)
                            {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::MemoryRetiredDbe(id),
                                    time,
                                    value.len().try_into().unwrap(),
                                );
                            }
                        }
                        NvidiaConfigStatistic::MemoryRetiredPending => {
                            if let Ok(value) = device.are_pages_pending_retired().map(|v| {
                                if v {
                                    1_u32
                                } else {
                                    0_u32
                                }
                            }) {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::MemoryRetiredDbe(id),
                                    time,
                                    value.into(),
                                );
                            }
                        }
                        NvidiaConfigStatistic::ProcessesCompute => {
                            if let Ok(value) = device.running_compute_processes_count() {
                                let _ = self.metrics().record_gauge(
                                    &NvidiaStatistic::ProcessesCompute(id),
                                    time,
                                    value.into(),
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
