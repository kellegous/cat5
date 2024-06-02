use std::error::Error;

use cat5::{update_data, DataDir};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value = "data")]
    data_dir: String,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    UpdateData(update_data::Args),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let client = reqwest::Client::new();
    let data_dir = DataDir::create(&client, &args.data_dir).await?;
    match args.command {
        Command::UpdateData(opts) => update_data::run(&data_dir, opts).await,
    }
}
