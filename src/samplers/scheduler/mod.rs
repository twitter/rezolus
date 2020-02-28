// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use async_trait::async_trait;
#[cfg(feature = "bpf")]
use bcc;
use chashmap::CHashMap;
use metrics::*;
#[cfg(feature = "perf")]
use perfcnt::*;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::runtime::Handle;

use crate::common::bpf::*;
use crate::common::*;
use crate::config::{Config, SamplerConfig};
use crate::samplers::Common;
use crate::Sampler;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[cfg(not(feature = "perf"))]
struct PerfCounter {}

#[allow(dead_code)]
pub struct Scheduler {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
    perf_counters: CHashMap<SchedulerStatistic, Vec<PerfCounter>>,
}

#[async_trait]
impl Sampler for Scheduler {
    type Statistic = SchedulerStatistic;
    fn new(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>) -> Result<Self, failure::Error> {
        let fault_tolerant = config.general().fault_tolerant();

        let perf_counters = CHashMap::new();
        if config.samplers().scheduler().enabled() && config.samplers().scheduler().perf_events() {
            #[cfg(feature = "perf")]
            {
                if let Ok(cores) = crate::common::hardware_threads() {
                    for statistic in config.samplers().scheduler().statistics().iter() {
                        if let Some(mut builder) = statistic.perf_counter_builder() {
                            let mut event_counters = Vec::new();
                            for core in 0..cores {
                                match builder.on_cpu(core as isize).for_all_pids().finish() {
                                    Ok(c) => event_counters.push(c),
                                    Err(e) => {
                                        debug!(
                                            "Failed to create PerfCounter for {:?}: {}",
                                            statistic, e
                                        );
                                    }
                                }
                            }
                            if event_counters.len() as u64 == cores {
                                trace!("Initialized PerfCounters for {:?}", statistic);
                                perf_counters.insert(*statistic, event_counters);
                            }
                        }
                    }
                } else {
                    if !fault_tolerant {
                        fatal!("failed to detect number of hardware threads");
                    } else {
                        error!("failed to detect number of hardware threads. skipping scheduler perf telemetry");
                    }
                }
            }
        }

        #[allow(unused_mut)]
        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common: Common::new(config, metrics),
            perf_counters,
        };

        if let Err(e) = sampler.initialize_bpf() {
            if !fault_tolerant {
                return Err(e);
            }
        }

        Ok(sampler)
    }

    fn spawn(config: Arc<Config>, metrics: Arc<Metrics<AtomicU32>>, handle: &Handle) {
        if let Ok(mut sampler) = Self::new(config.clone(), metrics) {
            handle.spawn(async move {
                loop {
                    let _ = sampler.sample().await;
                }
            });
        } else if !config.fault_tolerant() {
            fatal!("failed to initialize scheduler sampler");
        } else {
            error!("failed to initialize scheduler sampler");
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().scheduler()
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

        self.sample_proc_stat().await?;
        #[cfg(feature = "bpf")]
        self.sample_bpf().await?;
        #[cfg(feature = "perf")]
        self.sample_perf_counters().await?;

        Ok(())
    }

    fn summary(&self, statistic: &Self::Statistic) -> Option<Summary> {
        let precision = if statistic.bpf_table().is_some() {
            2
        } else {
            3
        };

        let max = if statistic.bpf_table().is_some() {
            SECOND
        } else {
            1_000_000
        };

        Some(Summary::histogram(
            max,
            precision,
            Some(self.general_config().window()),
        ))
    }
}

impl Scheduler {
    async fn sample_proc_stat(&self) -> Result<(), std::io::Error> {
        let file = File::open("/proc/stat").await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let time = time::precise_time_ns();
        while let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts[0] == "ctxt" && parts.len() == 2 {
                self.metrics().record_counter(
                    &SchedulerStatistic::ContextSwitches,
                    time,
                    parts[1].parse().unwrap_or(0),
                );
            }
            if parts[0] == "processes" && parts.len() == 2 {
                self.metrics().record_counter(
                    &SchedulerStatistic::ProcessesCreated,
                    time,
                    parts[1].parse().unwrap_or(0),
                );
            }
            if parts[0] == "procs_running" && parts.len() == 2 {
                self.metrics().record_gauge(
                    &SchedulerStatistic::ProcessesRunning,
                    time,
                    parts[1].parse().unwrap_or(0),
                );
            }
            if parts[0] == "procs_blocked" && parts.len() == 2 {
                self.metrics().record_gauge(
                    &SchedulerStatistic::ProcessesBlocked,
                    time,
                    parts[1].parse().unwrap_or(0),
                );
            }
        }

        Ok(())
    }

    #[cfg(feature = "bpf")]
    async fn sample_bpf(&mut self) -> Result<(), std::io::Error> {
        // sample bpf
        {
            if self.bpf_last.lock().unwrap().elapsed() >= self.general_config().window() {
                if let Some(ref bpf) = self.bpf {
                    let bpf = bpf.lock().unwrap();
                    let time = time::precise_time_ns();
                    for statistic in self.sampler_config().statistics() {
                        if let Some(table) = statistic.bpf_table() {
                            let mut table = (*bpf).inner.table(table);

                            for (&value, &count) in &map_from_table(&mut table) {
                                if count > 0 {
                                    self.metrics().record_distribution(
                                        statistic,
                                        time,
                                        value * MICROSECOND,
                                        count,
                                    );
                                }
                            }
                        }
                    }
                }
                *self.bpf_last.lock().unwrap() = Instant::now();
            }
        }

        Ok(())
    }

    #[cfg(feature = "perf")]
    async fn sample_perf_counters(&mut self) -> Result<(), std::io::Error> {
        let time = time::precise_time_ns();
        for stat in self.sampler_config().statistics() {
            if let Some(mut counters) = self.perf_counters.get_mut(stat) {
                let mut value = 0;
                for counter in counters.iter_mut() {
                    let count = match counter.read() {
                        Ok(c) => c,
                        Err(e) => {
                            debug!("Could not read perf counter for event {:?}: {}", stat, e);
                            0
                        }
                    };
                    value += count;
                }
                if value > 0 {
                    debug!("recording value for: {:?}", stat);
                }
                self.metrics().record_counter(stat, time, value);
            }
        }

        Ok(())
    }

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
                let mut bpf = bcc::core::BPF::new(code)?;

                // load + attach kprobes!
                let trace_run = bpf.load_kprobe("trace_run")?;
                let trace_ttwu_do_wakeup = bpf.load_kprobe("trace_ttwu_do_wakeup")?;
                let trace_wake_up_new_task = bpf.load_kprobe("trace_wake_up_new_task")?;

                bpf.attach_kprobe("finish_task_switch", trace_run)?;
                bpf.attach_kprobe("wake_up_new_task", trace_wake_up_new_task)?;
                bpf.attach_kprobe("ttwu_do_wakeup", trace_ttwu_do_wakeup)?;

                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
            }
        }

        Ok(())
    }
}
