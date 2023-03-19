mod app_data;
mod hardware;
mod message;
mod redis;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
};

use app_data::Cli;
use bus::Bus;
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

    let (redis_reader_tx, redis_reader_rx) = mpsc::channel();
    let (redis_writer_tx, redis_writer_rx) = mpsc::channel();
    let (port_handler_tx, port_handler_rx) = mpsc::channel();
    let bus = Arc::new(Mutex::new(Bus::new(100)));

    let redis_reader = {
        let exit_required = exit_required.clone();
        let config = config.clone();
        std::thread::spawn(move || {
            redis::reader::read_redis(exit_required, &config, redis_reader_tx)
        })
    };
    let redis_writer = {
        let exit_required = exit_required.clone();
        std::thread::spawn(move || {
            redis::writer::write_redis(exit_required, &config, redis_writer_rx)
        })
    };
    let port_handler = {
        let exit_required = exit_required.clone();
        let bus = bus.clone();
        std::thread::spawn(move || hardware::port::loop_query(exit_required, port_handler_tx, bus))
    };

    while !exit_required.load(Ordering::Acquire) {
        if let Ok(act) = redis_reader_rx.try_recv() {
            if let Ok(mut lock) = bus.lock() {
                lock.broadcast(act);
            }
        }
        if let Ok(msg) = port_handler_rx.try_recv() {
            debug!("Port reader: {:?}", msg);
            redis_writer_tx.send(msg)?;
        }
    }

    redis_reader
        .join()
        .map_err(|_| anyhow::anyhow!("Unexpected Redis reader exiting"))?;

    redis_writer
        .join()
        .map_err(|_| anyhow::anyhow!("Unexpected Redis writer exiting"))?;

    port_handler
        .join()
        .map_err(|_| anyhow::anyhow!("Unexpected port handler exiting"))?;

    Ok(())
}

fn main() {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_max_level(Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_max_level(Level::INFO)
            .init();
    }

    if let Err(e) = run() {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}
