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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataBase {
    pub queue: String,
    pub uri: String,
}
