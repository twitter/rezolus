// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod exposition;
mod general;
mod samplers;

use std::io::Read;
use std::net::{SocketAddr, ToSocketAddrs};

use clap::{App, Arg};
use rustcommon_logger::Level;
use serde_derive::*;

use crate::*;

use config::exposition::*;
pub use config::general::General;
use config::samplers::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    exposition: Exposition,
    #[serde(default)]
    general: General,
    #[serde(default)]
    samplers: Samplers,
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

    #[allow(dead_code)]
    pub fn exposition(&self) -> &Exposition {
        &self.exposition
    }

    pub fn general(&self) -> &General {
        &self.general
    }

    pub fn samplers(&self) -> &Samplers {
        &self.samplers
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
    fn bpf(&self) -> bool {
        false
    }
    fn enabled(&self) -> bool {
        false
    }
    fn interval(&self) -> Option<usize>;
    fn percentiles(&self) -> &[f64];
    fn perf_events(&self) -> bool {
        false
    }
    fn statistics(&self) -> Vec<<Self as config::SamplerConfig>::Statistic>;
}
