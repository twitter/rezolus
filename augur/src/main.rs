use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use augur::{Config, Profiler};
use clap::Parser;
use rustcommon_metrics::{Counter, DynBoxedMetric};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::exporter::serve_admin;

#[macro_use]
extern crate log;

mod exporter;

/// Whole system profiling daemon.
#[derive(Parser, Debug)]
#[command(
    author,
    version = concat!(env!("CARGO_PKG_VERSION"), "-", env!("AUGUR_RELEASE")),
    about,
    long_about = None
)]
struct Args {
    /// Path to a config file.
    #[arg(short, long)]
    config: PathBuf,
}

fn init_logger() {
    let mut env_var_name = concat!(env!("CARGO_PKG_NAME"), "_log").to_string();
    env_var_name.make_ascii_uppercase();

    if std::env::var_os(&env_var_name).is_none() {
        // Default log level is info since debug is rather noisy.
        // We also disable the log statement in zookeeper::io since it prints out the
        // cluster password by default.
        std::env::set_var(&env_var_name, "info");
    }

    env_logger::init_from_env(&env_var_name);
}

async fn run(args: Args) -> Result<(), anyhow::Error> {
    let mut config = Vec::new();
    File::open(&args.config)
        .await
        .context("Unable to open config file")?
        .read_to_end(&mut config)
        .await
        .with_context(|| format!("Unable to read contents of {}", args.config.display()))?;
    let config: Config = toml::from_slice(&config)
        .with_context(|| format!("Unable to parse contents of {}", args.config.display()))?;

    let _release = DynBoxedMetric::new(
        Counter::with_value(1),
        concat!("release/", env!("AUGUR_RELEASE")),
    );

    let _thread = tokio::spawn({
        let config = config.metrics.clone();
        async move {
            let address = format!("{}:{}", config.addr, config.port);
            let address: SocketAddr = match address.parse() {
                Ok(address) => address,
                Err(e) => {
                    error!("Unable to parse metrics server bind address: {}", e);
                    std::process::exit(2)
                }
            };

            if let Err(e) = serve_admin(address).await {
                error!("Admin server exited with an error: {}", e);
            }
        }
    });

    info!("Starting augur profiler!");

    let mut profiler = Profiler::new(config).await?;

    tokio::select! {
        res = profiler.profile() => res,
        res = tokio::signal::ctrl_c() => res.map_err(From::from)
    }
}

fn main() {
    init_logger();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(2)
        .build()
        .expect("Failed to build tokio runtime");

    if let Err(e) = runtime.block_on(run(Args::parse())) {
        error!("ERROR: {}", e);
        std::process::exit(1);
    }

    runtime.shutdown_timeout(Duration::from_secs(5));
}
