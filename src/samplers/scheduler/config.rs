// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::*;
use serde_derive::Deserialize;

use crate::config::SamplerConfig;

use super::stat::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerConfig {
    #[serde(default)]
    bpf: AtomicBool,
    #[serde(default)]
    enabled: AtomicBool,
    #[serde(default)]
    interval: Option<AtomicUsize>,
    #[serde(default = "default_percentiles")]
    percentiles: Vec<Percentile>,
    #[serde(default)]
    perf_events: AtomicBool,
    #[serde(default = "default_statistics")]
    statistics: Vec<SchedulerStatistic>,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            bpf: Default::default(),
            enabled: Default::default(),
            interval: Default::default(),
            percentiles: default_percentiles(),
            perf_events: Default::default(),
            statistics: default_statistics(),
        }
    }
}

fn default_percentiles() -> Vec<Percentile> {
    vec![
        Percentile::p1,
        Percentile::p10,
        Percentile::p50,
        Percentile::p90,
        Percentile::p99,
    ]
}

fn default_statistics() -> Vec<SchedulerStatistic> {
    vec![
        SchedulerStatistic::ContextSwitches,
        SchedulerStatistic::CpuMigrations,
        SchedulerStatistic::ProcessesBlocked,
        SchedulerStatistic::ProcessesCreated,
        SchedulerStatistic::ProcessesRunning,
        SchedulerStatistic::RunqueueLatency,
    ]
}

impl SamplerConfig for SchedulerConfig {
    type Statistic = SchedulerStatistic;

    fn bpf(&self) -> bool {
        self.bpf.load(Ordering::Relaxed)
    }

    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    fn interval(&self) -> Option<usize> {
        self.interval.as_ref().map(|v| v.load(Ordering::Relaxed))
    }

    fn percentiles(&self) -> &[Percentile] {
        &self.percentiles
    }

    fn perf_events(&self) -> bool {
        self.perf_events.load(Ordering::Relaxed)
    }

    fn statistics(&self) -> &[<Self as SamplerConfig>::Statistic] {
        &self.statistics
    }
}
