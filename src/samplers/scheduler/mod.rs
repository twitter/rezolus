// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use dashmap::DashMap;
#[cfg(feature = "perf")]
use perfcnt::*;
use rustcommon_metrics::*;
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

#[cfg(not(feature = "perf"))]
struct PerfCounter {}

#[allow(dead_code)]
pub struct Scheduler {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
    perf_counters: DashMap<SchedulerStatistic, Vec<PerfCounter>>,
}

#[async_trait]
impl Sampler for Scheduler {
    type Statistic = SchedulerStatistic;
    fn new(common: Common) -> Result<Self, failure::Error> {
        let fault_tolerant = common.config.general().fault_tolerant();

        let perf_counters = DashMap::new();
        if common.config.samplers().scheduler().enabled()
            && common.config.samplers().scheduler().perf_events()
        {
            #[cfg(feature = "perf")]
            {
                if let Ok(cores) = crate::common::hardware_threads() {
                    for statistic in common.config.samplers().scheduler().statistics().iter() {
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
                } else if !fault_tolerant {
                    fatal!("failed to detect number of hardware threads");
                } else {
                    error!("failed to detect number of hardware threads. skipping scheduler perf telemetry");
                }
            }
        }

        #[allow(unused_mut)]
        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
            perf_counters,
        };

        if let Err(e) = sampler.initialize_bpf() {
            if !fault_tolerant {
                return Err(e);
            }
        }

        Ok(sampler)
    }

    fn spawn(common: Common) {
        if let Ok(mut sampler) = Self::new(common.clone()) {
            common.handle.spawn(async move {
                loop {
                    let _ = sampler.sample().await;
                }
            });
        } else if !common.config.fault_tolerant() {
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

        self.map_result(self.sample_proc_stat().await)?;
        #[cfg(feature = "bpf")]
        self.map_result(self.sample_bpf())?;
        #[cfg(feature = "perf")]
        {
            let result = self.sample_perf_counters().await;
            self.map_result(result)?;
        }

        Ok(())
    }

    fn summary(&self, statistic: &Self::Statistic) -> Option<Summary> {
        let precision = if statistic.bpf_table().is_some() {
            2
        } else {
            3
        };

        Some(Summary::histogram(
            statistic.max(),
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
            match parts.get(0) {
                Some(&"ctxt") => {
                    self.metrics().record_counter(
                        &SchedulerStatistic::ContextSwitches,
                        time,
                        parts.get(1).map(|v| v.parse().unwrap_or(0)).unwrap_or(0),
                    );
                }
                Some(&"processes") => {
                    self.metrics().record_counter(
                        &SchedulerStatistic::ProcessesCreated,
                        time,
                        parts.get(1).map(|v| v.parse().unwrap_or(0)).unwrap_or(0),
                    );
                }
                Some(&"procs_running") => {
                    self.metrics().record_gauge(
                        &SchedulerStatistic::ProcessesRunning,
                        time,
                        parts.get(1).map(|v| v.parse().unwrap_or(0)).unwrap_or(0),
                    );
                }
                Some(&"procs_blocked") => {
                    self.metrics().record_gauge(
                        &SchedulerStatistic::ProcessesBlocked,
                        time,
                        parts.get(1).map(|v| v.parse().unwrap_or(0)).unwrap_or(0),
                    );
                }
                Some(_) | None => {}
            }
        }

        Ok(())
    }

    #[cfg(feature = "bpf")]
    fn sample_bpf(&self) -> Result<(), std::io::Error> {
        use crate::common::MICROSECOND;

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
                bcc::core::Kprobe::new()
                    .name("trace_run")
                    .function("finish_task_switch")
                    .attach(&mut bpf)?;
                bcc::core::Kprobe::new()
                    .name("trace_ttwu_do_wakeup")
                    .function("ttwu_do_wakeup")
                    .attach(&mut bpf)?;
                bcc::core::Kprobe::new()
                    .name("trace_wake_up_new_task")
                    .function("wake_up_new_task")
                    .attach(&mut bpf)?;
                    
                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
            }
        }

        Ok(())
    }
}
