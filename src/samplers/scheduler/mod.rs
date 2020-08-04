// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use rustcommon_metrics::*;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::common::bpf::*;
use crate::common::SAMPLE_PERIOD;
use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::Sampler;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Scheduler {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
    perf: Option<Arc<Mutex<Perf>>>,
    perf_last: Arc<Mutex<Instant>>,
}

#[async_trait]
impl Sampler for Scheduler {
    type Statistic = SchedulerStatistic;
    fn new(common: Common) -> Result<Self, failure::Error> {
        let fault_tolerant = common.config.general().fault_tolerant();

        #[allow(unused_mut)]
        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
            perf: None,
            perf_last: Arc::new(Mutex::new(Instant::now())),
        };

        if let Err(e) = sampler.initialize_bpf() {
            if !fault_tolerant {
                return Err(e);
            }
        }

        if let Err(e) = sampler.initialize_perf() {
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
        self.map_result(self.sample_perf())?;

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
                let mut bpf = bcc::BPF::new(code)?;

                // load + attach kprobes!
                bcc::Kprobe::new()
                    .handler("trace_run")
                    .function("finish_task_switch")
                    .attach(&mut bpf)?;
                bcc::Kprobe::new()
                    .handler("trace_ttwu_do_wakeup")
                    .function("ttwu_do_wakeup")
                    .attach(&mut bpf)?;
                bcc::Kprobe::new()
                    .handler("trace_wake_up_new_task")
                    .function("wake_up_new_task")
                    .attach(&mut bpf)?;

                self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
            }
        }

        Ok(())
    }

    #[cfg(feature = "perf")]
    fn perf_enabled(&self) -> bool {
        if self.sampler_config().perf_events() {
            for statistic in self.sampler_config().statistics() {
                if statistic.perf_config().is_some() {
                    return true;
                }
            }
        }
        false
    }

    fn initialize_perf(&mut self) -> Result<(), failure::Error> {
        #[cfg(feature = "perf")]
        {
            if self.enabled() && self.perf_enabled() {
                debug!("initializing perf");
                let code = include_str!("perf.c");
                let mut perf_bpf = bcc::BPF::new(code)?;

                for statistic in self.sampler_config().statistics() {
                    if let Some((name, event)) = statistic.perf_config() {
                        bcc::PerfEvent::new()
                            .handler(&format!("f_{}", name))
                            .event(event)
                            .sample_period(Some(SAMPLE_PERIOD))
                            .attach(&mut perf_bpf)?;
                    }
                }
                self.perf = Some(Arc::new(Mutex::new(Perf { inner: perf_bpf })));
            }
        }
        Ok(())
    }

    #[cfg(feature = "perf")]
    fn sample_perf(&self) -> Result<(), std::io::Error> {
        if self.perf_last.lock().unwrap().elapsed() >= self.general_config().window() {
            if let Some(ref perf) = self.perf {
                let perf = perf.lock().unwrap();
                let time = time::precise_time_ns();

                for statistic in self.sampler_config().statistics() {
                    if let Some((name, _)) = statistic.perf_config() {
                        let table = (*perf).inner.table(name);

                        // We only should have a single entry in the table with key = 0
                        let mut total = 0;
                        for entry in table.iter() {
                            let mut v = [0_u8; 8];
                            for (i, byte) in v.iter_mut().enumerate() {
                                *byte = *(entry.value).get(i).unwrap_or(&0);
                            }

                            total += u64::from_ne_bytes(v);
                        }

                        self.metrics().record_counter(statistic, time, total)
                    }
                }
            }
            *self.perf_last.lock().unwrap() = Instant::now();
        }
        Ok(())
    }
}
