use clap::Parser;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, default_value_t = String::from("data"),
help = "directory where data should be written")]
    data_dir: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let data_dir = Path::new(&args.data_dir);
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }

    println!("data_dir = {}", data_dir.display());

    Ok(())
}
