use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(author, version, about, long_about)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub database: DataBase,
    pub port: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataBase {
    pub reader_queue: String,
    pub writer_queue: String,
    pub reader_uri: String,
    pub writer_uri: String,
}
