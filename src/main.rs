// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[macro_use]
extern crate logger;

#[macro_use]
extern crate failure;

use std::sync::Arc;

use logger::Logger;
use metrics::*;
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

    // initialize signal handler
    debug!("initializing signal handler");
    let runnable = Arc::new(AtomicBool::new(true));
    let r = runnable.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    })
    .expect("Failed to set handler for SIGINT / SIGTERM");

    // initialize metrics
    debug!("initializing metrics");
    let metrics = Arc::new(Metrics::<AtomicU32>::new());

    // initialize async runtime
    debug!("initializing async runtime");
    let runtime = Builder::new()
        .threaded_scheduler()
        .enable_time()
        .core_threads(config.general().threads())
        .build()
        .unwrap();

    // spawn samplers
    debug!("spawning samplers");
    Cpu::spawn(config.clone(), metrics.clone(), runtime.handle());
    Disk::spawn(config.clone(), metrics.clone(), runtime.handle());
    Ext4::spawn(config.clone(), metrics.clone(), runtime.handle());
    Memory::spawn(config.clone(), metrics.clone(), runtime.handle());
    Network::spawn(config.clone(), metrics.clone(), runtime.handle());
    Rezolus::spawn(config.clone(), metrics.clone(), runtime.handle());
    Scheduler::spawn(config.clone(), metrics.clone(), runtime.handle());
    Softnet::spawn(config.clone(), metrics.clone(), runtime.handle());
    Tcp::spawn(config.clone(), metrics.clone(), runtime.handle());
    Udp::spawn(config.clone(), metrics.clone(), runtime.handle());
    Xfs::spawn(config.clone(), metrics.clone(), runtime.handle());

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
        config.general().count_suffix(),
    );

    while runnable.load(Ordering::Relaxed) {
        http.run();
    }

    Ok(())
}
