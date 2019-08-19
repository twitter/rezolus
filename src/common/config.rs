// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::*;
use clap::{App, Arg, ArgMatches};
use logger::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::process;
use std::time::Duration;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

pub const DEFAULT_SAMPLE_RATE_HZ: f64 = 1.0;
pub const DEFAULT_SAMPLER_TIMEOUT_MILLISECONDS: usize = 50;
pub const DEFAULT_MAX_SAMPLER_TIMEOUTS: usize = 5;
pub const DEFAULT_INTERVAL_SECONDS: usize = 60;

/// This struct contains the configuration of the agent.
#[derive(Clone)]
pub struct Config {
    /// the latching interval for stats
    interval: u64,
    /// sample rate for counters in Hz
    sample_rate: f64,
    /// the sampler timeout
    sampler_timeout: Duration,
    /// maximum consecutive sampler timeouts
    max_sampler_timeouts: usize,
    /// the listen address for the stats port
    listen: SocketAddr,
    /// the logging level
    loglevel: Level,
    /// memcache instance to instrument
    memcache: Option<SocketAddr>,
    /// flags for enabled statistics subsystems
    flags: Flags,
    /// the number of cores on the host
    cores: usize,
    /// an optional file to log stats to
    stats_log: Option<String>,
    /// flag to indicate Mesos sidecar mode
    sidecar: bool,
}

#[derive(Clone)]
/// `Flags` is a simple wrapper for a doubly-keyed `HashSet`
pub struct Flags {
    data: HashMap<String, HashSet<String>>,
}

impl Flags {
    /// Creates a new empty set of `Flags`
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Insert a `pkey`+`lkey` into the set
    pub fn insert(&mut self, pkey: &str, lkey: &str) {
        let mut entry = self.data.remove(pkey).unwrap_or_default();
        entry.insert(lkey.to_owned());
        self.data.insert(pkey.to_owned(), entry);
    }

    /// True if the set contains `pkey`+`lkey`
    pub fn contains(&self, pkey: &str, lkey: &str) -> bool {
        if let Some(entry) = self.data.get(pkey) {
            entry.get(lkey).is_some()
        } else {
            false
        }
    }

    /// True if the set contains the `pkey`
    pub fn contains_pkey(&self, pkey: &str) -> bool {
        self.data.get(pkey).is_some()
    }

    /// Remove a `pkey`+`lkey`
    pub fn remove(&mut self, pkey: &str, lkey: &str) {
        if let Some(entry) = self.data.get_mut(pkey) {
            entry.remove(lkey);
        }
    }

    /// Remove the `pkey` and all `lkey`s under it
    pub fn remove_pkey(&mut self, pkey: &str) {
        self.data.remove(pkey);
    }
}

impl Config {
    /// parse command line options and return `Config`
    pub fn new() -> Config {
        let matches = App::new(NAME)
            .version(VERSION)
            .author("Brian Martin <bmartin@twitter.com>")
            .about("high-resolution systems performance telemetry agent")
            .arg(
                Arg::with_name("listen")
                    .short("l")
                    .long("listen")
                    .required(true)
                    .takes_value(true)
                    .value_name("IP:PORT")
                    .help("Sets the listen address for metrics"),
            )
            .arg(
                Arg::with_name("verbose")
                    .short("v")
                    .long("verbose")
                    .multiple(true)
                    .help("Increase verbosity by one level. Can be used more than once"),
            )
            .arg(
                Arg::with_name("interval")
                    .long("interval")
                    .value_name("Seconds")
                    .help("Integration window duration and stats endpoint refresh time")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("sample-rate")
                    .long("sample-rate")
                    .value_name("Hertz")
                    .help("Sets the sampling frequency for the counters")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("sampler-timeout")
                    .long("sampler-timeout")
                    .value_name("MS")
                    .help("Sets the timeout for per-sampler execution")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("max-sampler-timeouts")
                    .long("max-sampler-timeouts")
                    .value_name("MS")
                    .help("Sets the maximum number of consecutive sampler timeouts")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("cpu")
                    .long("cpu")
                    .takes_value(true)
                    .multiple(true)
                    .possible_value("totals")
                    .help("Enable statistics from CPU subsystem"),
            )
            .arg(
                Arg::with_name("disk")
                    .long("disk")
                    .takes_value(true)
                    .multiple(true)
                    .possible_value("totals")
                    .help("Enable statistics from Disk subsystem"),
            )
            .arg(
                Arg::with_name("ebpf")
                    .long("ebpf")
                    .takes_value(true)
                    .multiple(true)
                    .possible_value("all")
                    .possible_value("block")
                    .possible_value("ext4")
                    .possible_value("scheduler")
                    .possible_value("xfs")
                    .help("Enable statistics from eBPF"),
            )
            .arg(
                Arg::with_name("network")
                    .long("network")
                    .takes_value(true)
                    .multiple(true)
                    .possible_value("totals")
                    .help("Enable statistics from Network subsystem"),
            )
            .arg(
                Arg::with_name("perf")
                    .long("perf")
                    .takes_value(true)
                    .multiple(true)
                    .possible_value("totals")
                    .possible_value("per-cgroup")
                    .help("Enable statistics from Perf Events subsystem"),
            )
            .arg(
                Arg::with_name("memcache")
                    .long("memcache")
                    .required(false)
                    .takes_value(true)
                    .value_name("IP:PORT")
                    .help("Connect to the given memcache server and produce stats"),
            )
            .arg(
                Arg::with_name("stats-log")
                    .long("stats-log")
                    .required(false)
                    .takes_value(true)
                    .value_name("LOG FILE")
                    .help("Enable logging of stats to file"),
            )
            .arg(
                Arg::with_name("sidecar")
                    .long("sidecar")
                    .required(false)
                    .help("Enables Mesos sidecar mode, instrumenting the container"),
            )
            .get_matches();

        let listen = matches
            .value_of("listen")
            .unwrap()
            .parse()
            .unwrap_or_else(|_| {
                println!("ERROR: listen address is malformed");
                process::exit(1);
            });

        let memcache = if let Some(sock) = matches.value_of("memcache") {
            let socket = sock.parse().unwrap_or_else(|_| {
                println!("ERROR: memcache address is malformed");
                process::exit(1);
            });
            Some(socket)
        } else {
            None
        };

        let sample_rate =
            parse_float_arg(&matches, "sample-rate").unwrap_or(DEFAULT_SAMPLE_RATE_HZ);
        let sampler_timeout = Duration::from_millis(
            parse_numeric_arg(&matches, "sampler-timeout")
                .unwrap_or(DEFAULT_SAMPLER_TIMEOUT_MILLISECONDS) as u64,
        );
        let max_sampler_timeouts = parse_numeric_arg(&matches, "max-sampler-timeouts")
            .unwrap_or(DEFAULT_MAX_SAMPLER_TIMEOUTS);
        let interval = parse_numeric_arg(&matches, "interval").unwrap_or(DEFAULT_INTERVAL_SECONDS)
            as u64
            * SECOND;
        let cores = hardware_threads().unwrap_or(1);

        let mut stats_enabled = Flags::new();
        for subsystem in &["cpu", "disk", "ebpf", "network", "perf"] {
            if let Some(values) = matches.values_of(subsystem) {
                let flags: Vec<&str> = values.collect();
                for flag in flags {
                    stats_enabled.insert(subsystem, flag);
                }
            }
        }

        let loglevel = match matches.occurrences_of("verbose") {
            0 => Level::Info,
            1 => Level::Debug,
            _ => Level::Trace,
        };

        let stats_log = matches
            .value_of("stats-log")
            .map(std::string::ToString::to_string);

        let sidecar = matches.is_present("sidecar");

        Config {
            cores,
            flags: stats_enabled,
            sample_rate,
            sampler_timeout,
            max_sampler_timeouts,
            interval,
            listen,
            loglevel,
            memcache,
            stats_log,
            sidecar,
        }
    }

    /// what interval should the stats library latch on
    pub fn interval(&self) -> u64 {
        self.interval
    }

    /// what frequency the stats should be sampled on
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    /// the timeout for sampler execution
    pub fn sampler_timeout(&self) -> Duration {
        self.sampler_timeout
    }

    /// maximum consecutive sampler timeouts
    pub fn max_sampler_timeouts(&self) -> usize {
        self.max_sampler_timeouts
    }

    /// get listen address
    pub fn listen(&self) -> SocketAddr {
        self.listen
    }

    /// get log level
    pub fn loglevel(&self) -> Level {
        self.loglevel
    }

    /// how many cores on the host?
    pub fn cores(&self) -> usize {
        self.cores
    }

    pub fn memcache(&self) -> Option<SocketAddr> {
        self.memcache
    }

    /// is a flag enabled for a subsystem?
    pub fn is_enabled(&self, subsystem: &str, flag: &str) -> bool {
        self.flags.contains(subsystem, flag)
    }

    pub fn is_subsystem_enabled(&self, subsystem: &str) -> bool {
        self.flags.contains_pkey(subsystem)
    }

    pub fn stats_log(&self) -> Option<String> {
        self.stats_log.clone()
    }
}

/// a helper function to parse a numeric argument by name from `ArgMatches`
fn parse_numeric_arg(matches: &ArgMatches, key: &str) -> Option<usize> {
    matches.value_of(key).map(|f| {
        f.parse().unwrap_or_else(|_| {
            println!("ERROR: could not parse {}", key);
            process::exit(1);
        })
    })
}

/// a helper function to parse a floating point argument by name from `ArgMatches`
fn parse_float_arg(matches: &ArgMatches, key: &str) -> Option<f64> {
    matches.value_of(key).map(|f| {
        f.parse().unwrap_or_else(|_| {
            println!("ERROR: could not parse {}", key);
            process::exit(1);
        })
    })
}

/// helper function to discover the number of hardware threads
pub fn hardware_threads() -> Result<usize, ()> {
    let path = "/sys/devices/system/cpu/present";
    let f = File::open(path).map_err(|e| error!("failed to open file ({:?}): {}", path, e))?;
    let mut f = BufReader::new(f);

    let mut line = String::new();
    f.read_line(&mut line)
        .map_err(|_| error!("failed to read line"))?;
    let line = line.trim();
    let a: Vec<&str> = line.split('-').collect();
    a.last()
        .unwrap_or(&"0")
        .parse::<usize>()
        .map_err(|e| error!("could not parse num cpus from file ({:?}): {}", path, e))
        .map(|i| i + 1)
}
