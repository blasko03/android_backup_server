mod api;
mod backup;

use clap::{Parser, Subcommand};
use api::server::start_server;
use crate::backup::device::Device;


#[derive(Parser)]
#[command(name = "backup")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    Server,
    FilesClean,
    ChunksClean,
    ConsistencyCheck
}

fn main() {
    env_logger::init();
    let uuid = "b699057f-8060-4c7b-96e5-ed62d0d491c3".to_string();
    let device = Device{uuid: uuid.clone()};
    let cli = Cli::parse();

    match cli.command {
        Commands::Server => {
            let _ = start_server();
        },
        Commands::FilesClean => {
            device.files().files_clean().unwrap();
        },
        Commands::ChunksClean => {
            device.chunks_clean().unwrap();
        },
        Commands::ConsistencyCheck => {
            device.consistency_check().unwrap();
        }
    }
}