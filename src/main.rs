// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[macro_use]
extern crate rustcommon_logger;

#[macro_use]
extern crate anyhow;

use rustcommon_atomics::{Atomic, Ordering};
use std::sync::Arc;

use rustcommon_atomics::AtomicBool;
use rustcommon_logger::Logger;
use rustcommon_metrics::*;
use tokio::runtime::Builder;

mod common;
mod config;
mod exposition;
mod samplers;

use common::*;
use config::Config;
use samplers::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get config
    let config = Arc::new(Config::new());

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

    let runnable = Arc::new(AtomicBool::new(true));
    let r = runnable.clone();

    // initialize signal handler
    debug!("initializing signal handler");
    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    })
    .expect("Failed to set handler for SIGINT / SIGTERM");

    // initialize metrics
    debug!("initializing metrics");
    let metrics = Arc::new(Metrics::<AtomicU64, AtomicU32>::new());

    // initialize async runtime
    debug!("initializing async runtime");
    let runtime = Arc::new(
        Builder::new_multi_thread()
            .enable_all()
            .worker_threads(config.general().threads())
            .max_blocking_threads(config.general().threads())
            .thread_name("rezolus-worker")
            .build()
            .unwrap(),
    );

    // spawn samplers
    debug!("spawning samplers");
    let common = Common::new(config.clone(), metrics.clone(), runtime);
    Cpu::spawn(common.clone());
    Disk::spawn(common.clone());
    Ext4::spawn(common.clone());
    Http::spawn(common.clone());
    Interrupt::spawn(common.clone());
    LibCall::spawn(common.clone());
    Memcache::spawn(common.clone());
    Memory::spawn(common.clone());
    PageCache::spawn(common.clone());
    Network::spawn(common.clone());
    Ntp::spawn(common.clone());
    Nvidia::spawn(common.clone());
    Rezolus::spawn(common.clone());
    Scheduler::spawn(common.clone());
    Softnet::spawn(common.clone());
    Tcp::spawn(common.clone());
    Udp::spawn(common.clone());
    Xfs::spawn(common);

    #[cfg(feature = "push_kafka")]
    {
        if config.exposition().kafka().enabled() {
            let mut kafka_producer =
                exposition::KafkaProducer::new(config.clone(), metrics.clone());
            let _ = std::thread::Builder::new()
                .name("kafka".to_string())
                .spawn(move || loop {
                    kafka_producer.run();
                });
        }
    }

    debug!("beginning stats exposition");
    let mut http = exposition::Http::new(
        config.listen().expect("no listen address"),
        metrics,
        config.general().reading_suffix(),
    );

    while runnable.load(Ordering::Relaxed) {
        http.run();
    }

    Ok(())
}
