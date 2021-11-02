// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
use std::io::SeekFrom;
use std::sync::{Arc, Mutex};
use std::time::*;

use async_trait::async_trait;
#[cfg(feature = "bpf")]
use bcc::perf_event::{Event, SoftwareEvent};
#[cfg(feature = "bpf")]
use bcc::{PerfEvent, PerfEventArray};
use rustcommon_metrics::{Source, Statistic};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};

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
    proc_stat: Option<File>,
    statistics: Vec<SchedulerStatistic>,
}

#[async_trait]
impl Sampler for Scheduler {
    type Statistic = SchedulerStatistic;
    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let fault_tolerant = common.config.general().fault_tolerant();
        let statistics = common.config().samplers().scheduler().statistics();

        #[allow(unused_mut)]
        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
            perf: None,
            proc_stat: None,
            statistics,
        };

        if sampler.sampler_config().enabled() {
            sampler.register();
        }

        if let Err(e) = sampler.initialize_bpf() {
            error!("{}", e);
            if !fault_tolerant {
                return Err(e);
            }
        }

        // we initialize perf last so we can delay
        if sampler.sampler_config().enabled() && sampler.sampler_config().perf_events() {
            #[cfg(feature = "bpf")]
            {
                if let Err(e) = sampler.initialize_bpf_perf() {
                    error!("{}", e);
                    if !fault_tolerant {
                        return Err(format_err!("bpf perf init failure: {}", e));
                    }
                }
            }
        }

        // delay by half the sample interval so that we land between perf
        // counter updates
        std::thread::sleep(std::time::Duration::from_micros(
            (1000 * sampler.interval()) as u64 / 2,
        ));

        Ok(sampler)
    }

    fn spawn(common: Common) {
        if common.config().samplers().scheduler().enabled() {
            if let Ok(mut sampler) = Self::new(common.clone()) {
                common.runtime().spawn(async move {
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

        let r = self.sample_proc_stat().await;
        self.map_result(r)?;
        #[cfg(feature = "bpf")]
        self.map_result(self.sample_bpf())?;

        Ok(())
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
            for statistic in &self.statistics {
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

    async fn sample_proc_stat(&mut self) -> Result<(), std::io::Error> {
        if self.proc_stat.is_none() {
            let file = File::open("/proc/stat").await?;
            self.proc_stat = Some(file);
        }

        if let Some(file) = &mut self.proc_stat {
            file.seek(SeekFrom::Start(0)).await?;
            let mut reader = BufReader::new(file);
            let mut line = String::new();
            let mut result = HashMap::new();

            while reader.read_line(&mut line).await? > 0 {
                let mut split = line.split_whitespace();
                if let Some(stat) = match split.next() {
                    Some("ctxt") => Some(SchedulerStatistic::ContextSwitches),
                    Some("processes") => Some(SchedulerStatistic::ProcessesCreated),
                    Some("procs_running") => Some(SchedulerStatistic::ProcessesRunning),
                    Some("procs_blocked") => Some(SchedulerStatistic::ProcessesBlocked),
                    _ => None,
                } {
                    let value = split.next().map(|v| v.parse().unwrap_or(0)).unwrap_or(0);
                    result.insert(stat, value);
                }
                line.clear();
            }
            let time = Instant::now();
            for statistic in &self.statistics {
                if let Some(value) = result.get(statistic) {
                    match statistic.source() {
                        Source::Counter => {
                            let _ = self.metrics().record_counter(statistic, time, *value);
                        }
                        Source::Gauge => {
                            let _ = self.metrics().record_gauge(statistic, time, *value);
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "bpf")]
    fn sample_bpf(&self) -> Result<(), std::io::Error> {
        use crate::common::MICROSECOND;

        // sample bpf
        {
            if self.bpf_last.lock().unwrap().elapsed()
                >= Duration::new(self.general_config().window() as u64, 0)
            {
                if let Some(ref bpf) = self.bpf {
                    let bpf = bpf.lock().unwrap();
                    let time = Instant::now();
                    for statistic in self.statistics.iter().filter(|s| s.bpf_table().is_some()) {
                        if let Ok(mut table) = (*bpf).inner.table(statistic.bpf_table().unwrap()) {
                            for (&value, &count) in &map_from_table(&mut table) {
                                if count > 0 {
                                    let _ = self.metrics().record_bucket(
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
            let time = Instant::now();
            for stat in self.statistics.iter().filter(|s| s.perf_table().is_some()) {
                if let Ok(table) = &(*bpf).inner.table(stat.perf_table().unwrap()) {
                    let map = crate::common::bpf::perf_table_to_map(table);
                    let mut total = 0;
                    for (_cpu, count) in map.iter() {
                        total += count;
                    }
                    let _ = self.metrics().record_counter(stat, time, total);
                }
            }
        }
        Ok(())
    }

    // checks that bpf is enabled in config and one or more bpf stats enabled
    #[cfg(feature = "bpf")]
    fn bpf_enabled(&self) -> bool {
        if self.sampler_config().bpf() {
            for statistic in &self.statistics {
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

    fn initialize_bpf(&mut self) -> Result<(), anyhow::Error> {
        #[cfg(feature = "bpf")]
        {
            if self.enabled() && self.bpf_enabled() {
                debug!("initializing bpf");
                // load the code and compile
                let code = include_str!("bpf.c");
                let code = code.replace("VALUE_TO_INDEX2_FUNC", include_str!("../../common/value_to_index2.c"));
                let mut bpf = bcc::BPF::new(&code)?;

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
