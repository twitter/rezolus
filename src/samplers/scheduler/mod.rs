// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
#[cfg(feature = "bpf")]
use bcc::perf_event::{Event, SoftwareEvent};
#[cfg(feature = "bpf")]
use bcc::{PerfEvent, PerfEventArray};
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

#[allow(dead_code)]
pub struct Scheduler {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
    perf: Option<Arc<Mutex<BPF>>>,
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
        };

        sampler.register();

        if let Err(e) = sampler.initialize_bpf() {
            if !fault_tolerant {
                return Err(e);
            }
        }

        // we initialize perf last so we can delay
        if sampler.sampler_config().enabled() && sampler.sampler_config().perf_events() {
            #[cfg(feature = "bpf")]
            {
                if let Err(e) = sampler.initialize_bpf_perf() {
                    if !fault_tolerant {
                        return Err(format_err!("bpf perf init failure: {}", e));
                    }
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_micros(
            (1000 * sampler.interval()) as u64 / 2,
        ));

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

        // we do perf sampling first, since it is time critical to keep it
        // between underlying counter updates
        #[cfg(feature = "bpf")]
        self.map_result(self.sample_bpf_perf_counters())?;

        self.map_result(self.sample_proc_stat().await)?;
        #[cfg(feature = "bpf")]
        self.map_result(self.sample_bpf())?;

        Ok(())
    }

    fn summary(&self, statistic: &Self::Statistic) -> Option<Summary> {
        let precision = match statistic {
            SchedulerStatistic::RunqueueLatency => 2,
            _ => 3,
        };

        Some(Summary::histogram(
            statistic.max(),
            precision,
            Some(self.general_config().window()),
        ))
    }
}

impl Scheduler {
    #[cfg(feature = "bpf")]
    fn initialize_bpf_perf(&mut self) -> Result<(), std::io::Error> {
        let cpus = crate::common::hardware_threads().unwrap();
        let interval = self.interval() as u64;
        let frequency = if interval > 1000 {
            1
        } else if interval == 0 {
            1
        } else {
            1000 / interval
        };

        let code = format!(
            "{}\n{}",
            format!("#define NUM_CPU {}", cpus),
            include_str!("perf.c").to_string()
        );
        if let Ok(mut bpf) = bcc::BPF::new(&code) {
            for statistic in self.sampler_config().statistics().iter() {
                if let Some(table) = statistic.perf_table() {
                    if let Some(event) = statistic.event() {
                        if PerfEventArray::new()
                            .table(&format!("{}_array", table))
                            .event(event)
                            .attach(&mut bpf)
                            .is_err()
                        {
                            if !self.common().config().general().fault_tolerant() {
                                fatal!("failed to initialize perf bpf for event: {:?}", event);
                            } else {
                                error!("failed to initialize perf bpf for event: {:?}", event);
                            }
                        }
                    }
                }
            }
            debug!("attaching software event to drive perf counter sampling");
            if PerfEvent::new()
                .handler("do_count")
                .event(Event::Software(SoftwareEvent::CpuClock))
                .sample_frequency(Some(frequency))
                .attach(&mut bpf)
                .is_err()
            {
                if !self.common().config().general().fault_tolerant() {
                    fatal!("failed to initialize perf bpf for cpu");
                } else {
                    error!("failed to initialize perf bpf for cpu");
                }
            }
            self.perf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
        } else if !self.common().config().general().fault_tolerant() {
            fatal!("failed to initialize perf bpf");
        } else {
            error!("failed to initialize perf bpf. skipping scheduler perf telemetry");
        }
        Ok(())
    }

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

    #[cfg(feature = "bpf")]
    fn sample_bpf_perf_counters(&self) -> Result<(), std::io::Error> {
        if let Some(ref bpf) = self.perf {
            let bpf = bpf.lock().unwrap();
            let time = time::precise_time_ns();
            for stat in self.sampler_config().statistics() {
                if let Some(table) = stat.perf_table() {
                    let map = crate::common::bpf::perf_table_to_map(&(*bpf).inner.table(table));
                    let mut total = 0;
                    for (_cpu, count) in map.iter() {
                        total += count;
                    }
                    self.metrics().record_counter(stat, time, total);
                }
            }
        }
        Ok(())
    }

    // checks that bpf is enabled in config and one or more bpf stats enabled
    #[cfg(feature = "bpf")]
    fn bpf_enabled(&self) -> bool {
        if self.sampler_config().bpf() {
            for statistic in self.sampler_config().statistics() {
                match statistic {
                    SchedulerStatistic::RunqueueLatency => {
                        return true;
                    }
                    _ => {}
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
}
