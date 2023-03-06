mod hardware;
mod redis_message;

use std::path::PathBuf;

use clap::{command, Parser};
use serde::{Deserialize, Serialize};

use crate::redis_message::ConnectResqust;

#[derive(Parser)]
#[command(author, version, about, long_about)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    database: DataBase,
}

#[derive(Serialize, Deserialize, Debug)]
struct DataBase {
    queue: String,
    uri: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = {
        let default_config_path = PathBuf::from("config.json");
        let config_path = cli.config.as_deref().unwrap_or(&default_config_path);
        println!("Value for config: {}", config_path.display());
        let config_file = std::fs::read_to_string(config_path)?;
        let config: AppConfig = serde_json::from_str(&config_file)?;
        println!("Config: {:?}", config);
        config
    };

    let client = redis::Client::open(config.database.uri)?;
    let mut con = client.get_connection()?;
    let mut pubsub = con.as_pubsub();
    pubsub.subscribe(config.database.queue)?;

    let ports = serialport::available_ports().expect("No ports found!");
    for p in ports {
        println!("{}", p.port_name);
        let mut port = serialport::new(p.port_name, 115_200)
            .open()
            .expect("Failed to open port");
        let mut readed = vec![];
        let mut serial_buf: Vec<u8> = vec![0; 1];
        loop {
            if let Ok(_) = port.read(serial_buf.as_mut_slice()) {
                readed.push(serial_buf[0]);
                if readed.last() == Some(&0x80) {
                    println!("readed: {:02X?}", readed);
                    let decoded = hardware::sm7bits::decode(&readed);
                    println!("decoded: {:02X?}", decoded);
                    readed.clear();
                }
            }
        }
    }

    // loop {
    //     let msg = pubsub.get_message()?;
    //     let payload: String = msg.get_payload()?;
    //     println!("channel '{}': \"{}\"", msg.get_channel_name(), payload);
    //     let parsed: ConnectResqust = serde_json::from_str(&payload)?;
    //     println!("parsed: {:?}", parsed);
    // }

    Ok(())
}
