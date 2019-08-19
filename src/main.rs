// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod common;
mod config;
mod samplers;
mod stats;

use crate::common::*;
use crate::config::Config;
use crate::samplers::*;

use logger::*;
use metrics::{Metrics, Reading};

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct Stats {
    sequential_timeouts: usize,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            sequential_timeouts: 0,
        }
    }
}

fn main() {
    // get config
    let config = Config::new();

    // initialize logging
    Logger::new()
        .label(common::NAME)
        .level(config.logging())
        .init()
        .expect("Failed to initialize logger");

    info!("----------");
    info!("{} {}", common::NAME, common::VERSION);
    info!("----------");
    debug!(
        "built: {} target: {}",
        env!("VERGEN_BUILD_TIMESTAMP"),
        env!("VERGEN_TARGET_TRIPLE")
    );
    debug!("host cores: {}", hardware_threads().unwrap_or(1));

    let metrics = Metrics::new();
    let recorder = metrics.recorder();
    let mut samplers = Vec::<(Box<dyn Sampler>, Stats)>::new();

    // register samplers
    if config.memcache().is_some() {
        info!("memcache proxy mode");
        if let Ok(Some(s)) = samplers::Memcache::new(&config, &recorder) {
            samplers.push((s, Stats::default()));
        }
    } else {
        if let Ok(Some(s)) = samplers::Cpu::new(&config, &recorder) {
            samplers.push((s, Stats::default()));
        }
        if let Ok(Some(s)) = samplers::Disk::new(&config, &recorder) {
            samplers.push((s, Stats::default()));
        }
        if let Ok(Some(s)) = samplers::Rezolus::new(&config, &recorder) {
            samplers.push((s, Stats::default()));
        }
        if let Ok(Some(s)) = samplers::Network::new(&config, &recorder) {
            samplers.push((s, Stats::default()));
        }
        #[cfg(feature = "ebpf")]
        {
            if config.ebpf().block() {
                if let Ok(Some(s)) = ebpf::Block::new(&config, &recorder) {
                    samplers.push((s, Stats::default()));
                }
            }
            if config.ebpf().ext4() {
                if let Ok(Some(s)) = ebpf::Ext4::new(&config, &recorder) {
                    samplers.push((s, Stats::default()));
                }
            }
            if config.ebpf().scheduler() {
                if let Ok(Some(s)) = ebpf::Scheduler::new(&config, &recorder) {
                    samplers.push((s, Stats::default()));
                }
            }
            if config.ebpf().xfs() {
                if let Ok(Some(s)) = ebpf::Xfs::new(&config, &recorder) {
                    samplers.push((s, Stats::default()));
                }
            }
        }
        #[cfg(feature = "perf")]
        {
            if let Ok(Some(s)) = samplers::Perf::new(&config, &recorder) {
                samplers.push((s, Stats::default()));
            }
        }
        if let Ok(Some(s)) = samplers::Softnet::new(&config, &recorder) {
            samplers.push((s, Stats::default()));
        }
    }

    let time = time::precise_time_ns();
    // calculate interval in nanoseconds between samples
    let sample_interval = config.general().interval() as u64 * MILLISECOND;

    // snapshot at 2Hz to prevent stale samples at 1Hz external sampling
    let snapshot_interval = SECOND / 2;

    // latching occurs at the config interval and resets latched histograms
    // as well as the min/max points
    let latch_interval = config.general().window().as_secs() as u64 * SECOND;

    let mut snapshot_time = time + snapshot_interval;
    let mut latch_time = time + latch_interval;

    let readings = Arc::new(Mutex::new(Vec::<Reading>::new()));

    let count_suffix = if config.memcache().is_none() {
        // running in Vex mode and we need a count suffix (all values must be leaf nodes)
        Some("count")
    } else {
        // running in memcache mode and must NOT have a count suffix
        // resulting in passthrough of original metric name
        None
    };

    let listen = config.listen().unwrap_or_else(|| {
        fatal!("no listen address");
    });
    let mut stats_http = stats::Http::new(listen, metrics.recorder(), count_suffix);
    let _ = thread::Builder::new()
        .name("http".to_string())
        .spawn(move || loop {
            stats_http.run();
        });

    if let Some(stats_log) = config.stats_log() {
        let mut stats_logger = stats::StatsLog::new(&stats_log, metrics.recorder(), count_suffix);
        let _ = thread::Builder::new()
            .name("logger".to_string())
            .spawn(move || loop {
                stats_logger.run();
            });
    }

    // let mut stats_stdout = stats::StatsLog::new(&recorder);

    let mut first_run = true;

    loop {
        debug!("Sampling...");
        let start = time::precise_time_ns();

        // sample each sampler
        let mut samplers_temp = Vec::new();
        for (mut sampler, mut stats) in samplers.drain(..) {
            let start = Instant::now();
            let result = sampler.sample();
            let stop = Instant::now();
            let runtime = stop - start;

            match result {
                Ok(_) => {
                    if !first_run && runtime.subsec_millis() as usize > config.general().timeout() {
                        stats.sequential_timeouts += 1;
                        if stats.sequential_timeouts >= config.general().max_timeouts() {
                            warn!(
                                "Sampler {} took over {} ms {} times sequentially. Failing the sampler",
                                sampler.name(),
                                config.general().timeout(),
                                stats.sequential_timeouts
                            );
                            sampler.deregister();
                        } else {
                            stats.sequential_timeouts = 0;
                            samplers_temp.push((sampler, stats));
                        }
                    } else {
                        samplers_temp.push((sampler, stats));
                    }
                }
                Err(_) => {
                    warn!(
                        "Sampler {} returned a fatal error. Failing the sampler",
                        sampler.name()
                    );
                    sampler.deregister();
                }
            }
        }
        samplers = samplers_temp;
        debug!("sampling complete");

        // take a snapshot if necessary
        if time >= snapshot_time {
            let current_readings = recorder.readings();
            let mut readings = readings.lock().unwrap();
            *readings = current_readings;
            snapshot_time += snapshot_interval;

            // clear any latched histograms and min/max if necessary
            if time >= latch_time {
                recorder.latch();
                latch_time += latch_interval;
            }
        }

        first_run = false;

        let stop = time::precise_time_ns();

        let sleep = sample_interval - (stop - start);
        debug!("sleep for: {} ns", sleep);
        thread::sleep(Duration::from_nanos(sleep));
    }
}
