// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod container;
mod cpu;
mod cpuidle;
mod disk;
mod ebpf;
mod general;
mod network;
mod perf;
mod softnet;

use self::container::Container;
use self::cpu::Cpu;
use self::cpuidle::CpuIdle;
use self::disk::Disk;
use self::ebpf::Ebpf;
use self::general::General;
use self::network::Network;
use self::perf::Perf;
use self::softnet::Softnet;

use crate::*;

use std::io::Read;
use std::net::{SocketAddr, ToSocketAddrs};

use clap::{App, Arg};
use logger::Level;
use serde_derive::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    container: Container,
    #[serde(default)]
    cpu: Cpu,
    #[serde(default)]
    cpuidle: CpuIdle,
    #[serde(default)]
    disk: Disk,
    #[serde(default)]
    ebpf: Ebpf,
    #[serde(default)]
    general: General,
    #[serde(default)]
    network: Network,
    #[serde(default)]
    perf: Perf,
    #[serde(default)]
    softnet: Softnet,
}

impl Config {
    /// parse command line options and return `Config`
    pub fn new() -> Config {
        let app = App::new(NAME)
            .version(VERSION)
            .author("Brian Martin <bmartin@twitter.com>")
            .about("High-Resolution Systems Performance Telemetry")
            .arg(
                Arg::with_name("config")
                    .long("config")
                    .value_name("FILE")
                    .help("TOML config file")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("verbose")
                    .short("v")
                    .long("verbose")
                    .help("Increase verbosity by one level. Can be used more than once")
                    .multiple(true),
            );

        let matches = app.get_matches();

        let mut config = if let Some(file) = matches.value_of("config") {
            Config::load_from_file(file)
        } else {
            println!("NOTE: using builtin base configuration");
            Default::default()
        };

        match matches.occurrences_of("verbose") {
            0 => {} // don't do anything, default is Info
            1 => {
                if config.general.logging() == Level::Info {
                    config.general.set_logging(Level::Debug);
                }
            }
            _ => config.general.set_logging(Level::Trace),
        }

        config
    }

    /// get listen address
    pub fn listen(&self) -> Option<SocketAddr> {
        self.general
            .listen()
            .map(|v| v.to_socket_addrs().unwrap().next().unwrap())
    }

    /// get logging level
    pub fn logging(&self) -> Level {
        self.general.logging()
    }

    pub fn memcache(&self) -> Option<SocketAddr> {
        self.general
            .memcache()
            .map(|v| v.to_socket_addrs().unwrap().next().unwrap())
    }

    pub fn stats_log(&self) -> Option<String> {
        self.general.stats_log()
    }

    pub fn container(&self) -> &Container {
        &self.container
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn cpuidle(&self) -> &CpuIdle {
        &self.cpuidle
    }

    pub fn disk(&self) -> &Disk {
        &self.disk
    }

    pub fn general(&self) -> &General {
        &self.general
    }

    pub fn interval(&self) -> usize {
        self.general().interval()
    }

    pub fn network(&self) -> &Network {
        &self.network
    }

    #[cfg(feature = "perf")]
    pub fn perf(&self) -> &Perf {
        &self.perf
    }

    pub fn softnet(&self) -> &Softnet {
        &self.softnet
    }

    #[allow(dead_code)]
    pub fn ebpf(&self) -> &Ebpf {
        &self.ebpf
    }

    fn load_from_file(filename: &str) -> Config {
        let mut file = std::fs::File::open(filename).expect("failed to open workload file");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("failed to read");
        let toml = toml::from_str(&content);
        match toml {
            Ok(toml) => toml,
            Err(e) => {
                println!("Failed to parse TOML config: {}", filename);
                println!("{}", e);
                std::process::exit(1);
            }
        }
    }
}

pub trait SamplerConfig {
    fn enabled(&self) -> bool;
    fn interval(&self) -> Option<usize>;
}
