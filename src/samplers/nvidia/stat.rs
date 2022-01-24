// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::metrics::*;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumIter,
    EnumString,
    Eq,
    IntoStaticStr,
    PartialEq,
    Hash,
    Serialize,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum NvidiaConfigStatistic {
    #[strum(serialize = "gpu/temperature")]
    GpuTemperature,
    #[strum(serialize = "memory/ecc/sbe")]
    MemoryEccSbe,
    #[strum(serialize = "memory/ecc/dbe")]
    MemoryEccDbe,
    #[strum(serialize = "memory/ecc/enabled")]
    MemoryEccEnabled,
    #[strum(serialize = "power/usage")]
    PowerUsage,
    #[strum(serialize = "power/limit")]
    PowerLimit,
    #[strum(serialize = "energy/consumption")]
    EnergyConsumption,
    #[strum(serialize = "clock/sm/current")]
    ClockSMCurrent,
    #[strum(serialize = "clock/memory/current")]
    ClockMemoryCurrent,
    #[strum(serialize = "pcie/replay")]
    PcieReplay,
    #[strum(serialize = "pcie/rx/throughput")]
    PcieRxThroughput,
    #[strum(serialize = "pcie/tx/throughput")]
    PcieTxThroughput,
    #[strum(serialize = "gpu/utilization")]
    GpuUtilization,
    #[strum(serialize = "memory/utilization")]
    MemoryUtilization,
    #[strum(serialize = "decoder/utilization")]
    DecoderUtilization,
    #[strum(serialize = "encoder/utilization")]
    EncoderUtilization,
    #[strum(serialize = "memory/fb/free")]
    MemoryFbFree,
    #[strum(serialize = "memory/fb/total")]
    MemoryFbTotal,
    #[strum(serialize = "memory/fb/used")]
    MemoryFbUsed,
    #[strum(serialize = "memory/retired/sbe")]
    MemoryRetiredSbe,
    #[strum(serialize = "memory/retired/dbe")]
    MemoryRetiredDbe,
    #[strum(serialize = "memory/retired/pending")]
    MemoryRetiredPending,
    #[strum(serialize = "processes/compute")]
    ProcessesCompute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum NvidiaStatistic {
    GpuTemperature(u32),
    MemoryEccSbe(u32),
    MemoryEccDbe(u32),
    MemoryEccEnabled(u32),
    PowerUsage(u32),
    PowerLimit(u32),
    EnergyConsumption(u32),
    ClockSMCurrent(u32),
    ClockMemoryCurrent(u32),
    PcieReplay(u32),
    PcieRxThroughput(u32),
    PcieTxThroughput(u32),
    GpuUtilization(u32),
    MemoryUtilization(u32),
    DecoderUtilization(u32),
    EncoderUtilization(u32),
    MemoryFbFree(u32),
    MemoryFbTotal(u32),
    MemoryFbUsed(u32),
    MemoryRetiredSbe(u32),
    MemoryRetiredDbe(u32),
    MemoryRetiredPending(u32),
    ProcessesCompute(u32),
}

impl Statistic for NvidiaStatistic {
    // TODO(bmartin): this should be cleaned up once we have scoped metrics
    fn name(&self) -> &str {
        match self {
            NvidiaStatistic::GpuTemperature(id) => match id {
                0 => "nvidia/gpu_0/gpu/temperature",
                1 => "nvidia/gpu_1/gpu/temperature",
                _ => "nvidia/gpu_unknown/gpu/temperature",
            },
            NvidiaStatistic::MemoryEccEnabled(id) => match id {
                0 => "nvidia/gpu_0/memory/ecc/enabled",
                1 => "nvidia/gpu_1/memory/ecc/enabled",
                _ => "nvidia/gpu_unknown/memory/ecc/enabled",
            },
            NvidiaStatistic::MemoryEccSbe(id) => match id {
                0 => "nvidia/gpu_0/memory/ecc/sbe",
                1 => "nvidia/gpu_1/memory/ecc/sbe",
                _ => "nvidia/gpu_unknown/memory/ecc/sbe",
            },
            NvidiaStatistic::MemoryEccDbe(id) => match id {
                0 => "nvidia/gpu_0/memory/ecc/dbe",
                1 => "nvidia/gpu_1/memory/ecc/dbe",
                _ => "nvidia/gpu_unknown/memory/ecc/dbe",
            },
            NvidiaStatistic::PowerUsage(id) => match id {
                0 => "nvidia/gpu_0/power/usage",
                1 => "nvidia/gpu_1/power/usage",
                _ => "nvidia/gpu_unknown/power/usage",
            },
            NvidiaStatistic::PowerLimit(id) => match id {
                0 => "nvidia/gpu_0/power/limit",
                1 => "nvidia/gpu_1/power/limit",
                _ => "nvidia/gpu_unknown/power/limit",
            },
            NvidiaStatistic::EnergyConsumption(id) => match id {
                0 => "nvidia/gpu_0/energy/consumption",
                1 => "nvidia/gpu_1/energy/consumption",
                _ => "nvidia/gpu_unknown/energy/consumption",
            },
            NvidiaStatistic::ClockSMCurrent(id) => match id {
                0 => "nvidia/gpu_0/clock/sm/current",
                1 => "nvidia/gpu_1/clock/sm/current",
                _ => "nvidia/gpu_unknown/clock/sm/current",
            },
            NvidiaStatistic::ClockMemoryCurrent(id) => match id {
                0 => "nvidia/gpu_0/clock/memory/current",
                1 => "nvidia/gpu_1/clock/memory/current",
                _ => "nvidia/gpu_unknown/clock/memory/current",
            },
            NvidiaStatistic::PcieReplay(id) => match id {
                0 => "nvidia/gpu_0/pcie/replay",
                1 => "nvidia/gpu_1/pcie/replay",
                _ => "nvidia/gpu_unknown/pcie/replay",
            },
            NvidiaStatistic::PcieRxThroughput(id) => match id {
                0 => "nvidia/gpu_0/pcie/rx/throughput",
                1 => "nvidia/gpu_1/pcie/rx/throughput",
                _ => "nvidia/gpu_unknown/pcie/rx/throughput",
            },
            NvidiaStatistic::PcieTxThroughput(id) => match id {
                0 => "nvidia/gpu_0/pcie/tx/throughput",
                1 => "nvidia/gpu_1/pcie/tx/throughput",
                _ => "nvidia/gpu_unknown/pcie/tx/throughput",
            },
            NvidiaStatistic::GpuUtilization(id) => match id {
                0 => "nvidia/gpu_0/gpu/utilization",
                1 => "nvidia/gpu_1/gpu/utilization",
                _ => "nvidia/gpu_unknown/gpu/utilization",
            },
            NvidiaStatistic::MemoryUtilization(id) => match id {
                0 => "nvidia/gpu_0/memory/utilization",
                1 => "nvidia/gpu_1/memory/utilization",
                _ => "nvidia/gpu_unknown/memory/utilization",
            },
            NvidiaStatistic::DecoderUtilization(id) => match id {
                0 => "nvidia/gpu_0/decoder/utilization",
                1 => "nvidia/gpu_1/decoder/utilization",
                _ => "nvidia/gpu_unknown/decoder/utilization",
            },
            NvidiaStatistic::EncoderUtilization(id) => match id {
                0 => "nvidia/gpu_0/encoder/utilization",
                1 => "nvidia/gpu_1/encoder/utilization",
                _ => "nvidia/gpu_unknown/encoder/utilization",
            },
            NvidiaStatistic::MemoryFbFree(id) => match id {
                0 => "nvidia/gpu_0/memory/fb/free",
                1 => "nvidia/gpu_1/memory/fb/free",
                _ => "nvidia/gpu_unknown/memory/fb/free",
            },
            NvidiaStatistic::MemoryFbTotal(id) => match id {
                0 => "nvidia/gpu_0/memory/fb/total",
                1 => "nvidia/gpu_1/memory/fb/total",
                _ => "nvidia/gpu_unknown/memory/fb/total",
            },
            NvidiaStatistic::MemoryFbUsed(id) => match id {
                0 => "nvidia/gpu_0/memory/fb/used",
                1 => "nvidia/gpu_1/memory/fb/used",
                _ => "nvidia/gpu_unknown/memory/fb/used",
            },
            NvidiaStatistic::MemoryRetiredSbe(id) => match id {
                0 => "nvidia/gpu_0/memory/retired/sbe",
                1 => "nvidia/gpu_1/memory/retired/sbe",
                _ => "nvidia/gpu_unknown/memory/retired/sbe",
            },
            NvidiaStatistic::MemoryRetiredDbe(id) => match id {
                0 => "nvidia/gpu_0/memory/retired/dbe",
                1 => "nvidia/gpu_1/memory/retired/dbe",
                _ => "nvidia/gpu_unknown/memory/retired/dbe",
            },
            NvidiaStatistic::MemoryRetiredPending(id) => match id {
                0 => "nvidia/gpu_0/memory/retired/pending",
                1 => "nvidia/gpu_1/memory/retired/pending",
                _ => "nvidia/gpu_unknown/memory/retired/pending",
            },
            NvidiaStatistic::ProcessesCompute(id) => match id {
                0 => "nvidia/gpu_0/processes/compute",
                1 => "nvidia/gpu_1/processes/compute",
                _ => "nvidia/gpu_unknown/processes/compute",
            },
        }
    }

    fn source(&self) -> Source {
        match self {
            Self::MemoryEccSbe(_)
            | Self::MemoryEccDbe(_)
            | Self::EnergyConsumption(_)
            | Self::MemoryRetiredDbe(_)
            | Self::MemoryRetiredSbe(_)
            | Self::PcieReplay(_) => Source::Counter,
            _ => Source::Gauge,
        }
    }
}
