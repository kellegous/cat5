use cat5::hurdat2::{Status, Storm, StormIter};
use cat5::{noaa, DataDir};
use clap::Parser;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
use std::io;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, default_value_t = String::from("data"),
        help = "directory where data should be written")]
    data_dir: String,

    #[clap(long, default_value_t = noaa::hurdat2_url().to_owned(),
        help = "NOAA URL to download hurdat2 data")]
    hurdat2_url: String,
}

fn is_hurricane(storm: &Storm) -> bool {
    storm
        .track()
        .iter()
        .any(|e| e.status() == Status::Hurricane)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let data_dir = DataDir::at(&args.data_dir)?;
    let mut r = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(data_dir.download("hurdat2.csv", &args.hurdat2_url)?);
    let hurricanes = StormIter::new(r.records()).filter(|r| match r {
        Ok(s) => is_hurricane(s),
        _ => true,
    });
    for storm in hurricanes {
        let storm = storm?;
        println!(
            "{} / {} / {}",
            storm.id(),
            storm.name().unwrap_or("??"),
            storm.track().len()
        );
    }
    // for storm in StormIter::new(r.records()) {
    //     let storm = storm?;
    //     println!(
    //         "{} / {} / {}",
    //         storm.id(),
    //         storm.name().unwrap_or("??"),
    //         storm.track().len()
    //     );
    // }
    // let data_dir = Path::new(&args.data_dir);
    // if !data_dir.exists() {
    //     fs::create_dir_all(&data_dir)?;
    // }

    // let hurdat2_data_file = data_dir.join("hurdat2.csv");
    // ensure_download_to(&hurdat2_data_file, &args.hurdat2_url)?;
    // // /compute_hash(&hurdat2_data_file)?;

    // println!("data_dir = {}", data_dir.display());
    println!("hurdat2_url = {}", args.hurdat2_url);

    Ok(())
}
