// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![deny(clippy::all)]
#![allow(clippy::new_ret_no_self)]

#[macro_use]
extern crate logger;

mod common;
mod config;
mod samplers;
mod stats;

use crate::common::*;
use crate::config::Config;
use crate::samplers::*;

use atomics::{AtomicBool, AtomicPrimitive, Ordering};
use logger::Logger;
use metrics::{Metrics, Reading};
use slab::Slab;
use timer::Wheel;

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

#[allow(clippy::cognitive_complexity)]
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

    // initialize signal handler
    let runnable = Arc::new(AtomicBool::new(true));
    let r = runnable.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Failed to set handler for SIGINT / SIGTERM");

    // initialize metrics
    let metrics = Metrics::new();
    let mut samplers = Slab::<(Box<dyn Sampler>, Stats)>::new();
    let mut timer = Wheel::<usize>::new(1000);

    let config = Arc::new(config);

    // register samplers
    if config.memcache().is_some() {
        info!("memcache proxy mode");
        match samplers::Memcache::new(config.clone(), metrics.clone()) {
            Ok(Some(s)) => {
                let token = samplers.insert((s, Stats::default()));
                timer.add(token, config.interval());
            }
            Ok(None) => {}
            Err(e) => {
                if !config.fault_tolerant() {
                    fatal!("Error initializing: {}", e);
                }
            }
        }
    } else {
        for sampler in vec![
            samplers::Container::new(config.clone(), metrics.clone()),
            samplers::Cpu::new(config.clone(), metrics.clone()),
            samplers::CpuIdle::new(config.clone(), metrics.clone()),
            samplers::Disk::new(config.clone(), metrics.clone()),
            samplers::Network::new(config.clone(), metrics.clone()),
            samplers::Rezolus::new(config.clone(), metrics.clone()),
            samplers::Softnet::new(config.clone(), metrics.clone()),
        ] {
            match sampler {
                Ok(Some(s)) => {
                    let token = samplers.insert((s, Stats::default()));
                    timer.add(token, config.interval());
                }
                Ok(None) => {}
                Err(e) => {
                    if !config.fault_tolerant() {
                        fatal!("Error initializing: {}", e);
                    }
                }
            }
        }
        #[cfg(feature = "ebpf")]
        {
            for sampler in vec![
                ebpf::Block::new(config.clone(), metrics.clone()),
                ebpf::Ext4::new(config.clone(), metrics.clone()),
                ebpf::Network::new(config.clone(), metrics.clone()),
                ebpf::Scheduler::new(config.clone(), metrics.clone()),
                ebpf::Tcp::new(config.clone(), metrics.clone()),
                ebpf::Xfs::new(config.clone(), metrics.clone()),
            ] {
                match sampler {
                    Ok(Some(s)) => {
                        let token = samplers.insert((s, Stats::default()));
                        timer.add(token, config.interval());
                    }
                    Ok(None) => {}
                    Err(e) => {
                        if !config.fault_tolerant() {
                            fatal!("Error initializing: {}", e);
                        }
                    }
                }
            }
        }
        #[cfg(feature = "perf")]
        {
            match samplers::Perf::new(config.clone(), metrics.clone()) {
                Ok(Some(s)) => {
                    let token = samplers.insert((s, Stats::default()));
                    timer.add(token, config.interval());
                }
                Ok(None) => {}
                Err(e) => {
                    if !config.fault_tolerant() {
                        fatal!("Error initializing: {}", e);
                    }
                }
            }
        }
    }

    let time = time::precise_time_ns();

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
        // resulting in pass-through of original metric name
        None
    };

    let listen = config.listen().unwrap_or_else(|| {
        fatal!("no listen address");
    });
    let mut stats_http = stats::Http::new(listen, metrics.clone(), count_suffix);
    let _ = thread::Builder::new()
        .name("http".to_string())
        .spawn(move || loop {
            stats_http.run();
        });

    if let Some(stats_log) = config.stats_log() {
        let mut stats_logger = stats::StatsLog::new(&stats_log, metrics.clone(), count_suffix);
        let _ = thread::Builder::new()
            .name("logger".to_string())
            .spawn(move || loop {
                stats_logger.run();
            });
    }

    if config.kafka().enabled() {
        let mut kafka_producer =
            stats::KafkaProducer::new(config.clone(), metrics.clone(), count_suffix);
        let _ = thread::Builder::new()
            .name("kafka".to_string())
            .spawn(move || loop {
                kafka_producer.run();
            });
    }

    let mut first_run = true;
    let mut t0 = time::precise_time_ns();

    while runnable.load(Ordering::Relaxed) {
        let t1 = time::precise_time_ns();
        let ticks = (t1 - t0) / MILLISECOND;
        t0 += ticks * MILLISECOND;
        let to_sample = timer.tick(ticks as usize);
        trace!(
            "ticked: {} ms and sampling: {} samplers",
            ticks,
            to_sample.len()
        );
        for token in to_sample {
            let start = Instant::now();
            let (sampler, stats) = samplers.get_mut(token).unwrap();
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
                            timer.add(token, sampler.interval());
                        }
                    } else {
                        timer.add(token, sampler.interval());
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
        debug!("sampling complete");

        // take a snapshot if necessary
        if time >= snapshot_time {
            let current_readings = metrics.readings();
            let mut readings = readings.lock().unwrap();
            *readings = current_readings;
            snapshot_time += snapshot_interval;

            // clear any latched histograms and min/max if necessary
            if time >= latch_time {
                metrics.latch();
                latch_time += latch_interval;
            }
        }

        first_run = false;

        let sleep = timer.next_timeout().unwrap_or(1000) as u64 * MILLISECOND;
        debug!("sleep for: {} ns", sleep);
        thread::sleep(Duration::from_nanos(sleep));
    }
}
