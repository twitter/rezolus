// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod general;
mod kafka;

use metrics::Percentile;
use samplers::cpu::CpuConfig;
use samplers::cpuidle::CpuidleConfig;
use samplers::disk::DiskConfig;
use samplers::ext4::Ext4Config;
use samplers::memcache::MemcacheConfig;
use samplers::memory::MemoryConfig;
use samplers::network::NetworkConfig;
use samplers::perf::PerfConfig;
use samplers::rezolus::RezolusConfig;
use samplers::scheduler::SchedulerConfig;
use samplers::softnet::SoftnetConfig;
use samplers::tcp::TcpConfig;
use samplers::udp::UdpConfig;
use samplers::xfs::XfsConfig;

pub use self::general::General;
use self::kafka::Kafka;

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
    cpu: CpuConfig,
    #[serde(default)]
    cpuidle: CpuidleConfig,
    #[serde(default)]
    disk: DiskConfig,
    #[serde(default)]
    ext4: Ext4Config,
    #[serde(default)]
    general: General,
    #[serde(default)]
    kafka: Kafka,
    #[serde(default)]
    network: Network,
    #[serde(default)]
    tcp: TcpConfig,
    #[serde(default)]
    udp: UdpConfig,
    #[serde(default)]
    xfs: XfsConfig,
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

    pub fn cpu(&self) -> &CpuConfig {
        &self.cpu
    }

    pub fn cpuidle(&self) -> &CpuidleConfig {
        &self.cpuidle
    }

    pub fn disk(&self) -> &DiskConfig {
        &self.disk
    }

    pub fn ext4(&self) -> &Ext4Config {
        &self.ext4
    }

    pub fn general(&self) -> &General {
        &self.general
    }

    pub fn kafka(&self) -> &Kafka {
        &self.kafka
    }

    pub fn memcache(&self) -> &MemcacheConfig {
        &self.memcache
    }

    pub fn memory(&self) -> &MemoryConfig {
        &self.memory
    }

    pub fn network(&self) -> &NetworkConfig {
        &self.network
    }

    #[cfg(feature = "perf")]
    pub fn perf(&self) -> &PerfConfig {
        &self.perf
    }

    pub fn rezolus(&self) -> &RezolusConfig {
        &self.rezolus
    }

    pub fn scheduler(&self) -> &SchedulerConfig {
        &self.scheduler
    }

    pub fn softnet(&self) -> &SoftnetConfig {
        &self.softnet
    }

    pub fn tcp(&self) -> &TcpConfig {
        &self.tcp
    }

    pub fn udp(&self) -> &UdpConfig {
        &self.udp
    }

    pub fn xfs(&self) -> &XfsConfig {
        &self.xfs
    }

    pub fn fault_tolerant(&self) -> bool {
        self.general().fault_tolerant()
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
    type Statistic;
    fn ebpf(&self) -> bool {
        false
    }
    fn enabled(&self) -> bool {
        false
    }
    fn interval(&self) -> Option<usize>;
    fn percentiles(&self) -> &[Percentile];
    fn statistics(&self) -> &[<Self as config::SamplerConfig>::Statistic];
}
