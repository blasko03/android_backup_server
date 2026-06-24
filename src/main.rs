mod api;
mod backup;

use crate::backup::device::Device;
use api::server::start_server;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "backup")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    Server,
    FilesClean(FilesCleanArgs),
    ChunksClean(ChunksCleanArgs),
    ConsistencyCheck(DeviceArgs),
}

#[derive(Args)]
struct DeviceArgs {
    device: String,
}
#[derive(Args)]
struct ChunksCleanArgs {
    #[command(flatten)]
    device: DeviceArgs,
    #[arg(long)]
    report_only: bool,
}

#[derive(Args)]
struct FilesCleanArgs {
    #[command(flatten)]
    device: DeviceArgs,
    #[arg(long)]
    report_only: bool,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Server => {
            let _ = start_server();
        }
        Commands::FilesClean(args) => {
            let device = Device {
                uuid: args.device.device,
            };
            device.files().files_clean(args.report_only).unwrap();
        }
        Commands::ChunksClean(args) => {
            let device = Device {
                uuid: args.device.device,
            };
            device.chunks_clean(args.report_only).unwrap();
        }
        Commands::ConsistencyCheck(args) => {
            let device = Device { uuid: args.device };
            device.consistency_check().unwrap();
        }
    }
}
