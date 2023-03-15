mod app_data;
mod hardware;
mod redis;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use app_data::Cli;
use clap::Parser;
use tracing::{debug, error, Level};

use crate::app_data::AppConfig;

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = {
        let default_config_path = PathBuf::from("config.json");
        let config_path = cli.config.as_deref().unwrap_or(&default_config_path);
        debug!("Value for config: {}", config_path.display());

        let config_file = std::fs::read_to_string(config_path)?;
        let config: AppConfig = serde_json::from_str(&config_file)?;
        debug!("Config: {:?}", config);

        config
    };

    let exit_required = Arc::new(AtomicBool::new(false));

    {
        let exit_required = exit_required.clone();
        ctrlc::set_handler(move || {
            exit_required.store(true, Ordering::Release);
        })?;
    }

    let redis_reader = {
        let exit_required = exit_required.clone();
        std::thread::spawn(move || {
            if let Err(e) = redis::reader::read_redis(exit_required, &config) {
                error!("Error: {}", e);
            }
        })
    };
    let port_handler = {
        let exit_required = exit_required.clone();
        std::thread::spawn(move || {
            if let Err(e) = hardware::port::loop_query(exit_required) {
                error!("Error: {}", e);
            }
        })
    };

    redis_reader
        .join()
        .map_err(|_| anyhow::anyhow!("Unexpected Redis reader exiting"))?;

    port_handler
        .join()
        .map_err(|_| anyhow::anyhow!("Unexpected port handler exiting"))?;

    Ok(())
}

fn main() {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .without_time()
            .with_max_level(Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .without_time()
            .with_max_level(Level::INFO)
            .init();
    }

    if let Err(e) = run() {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}
