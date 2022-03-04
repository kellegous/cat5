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

fn ensure_download_to<P: AsRef<Path>>(dst: P, url: &str) -> Result<(), Box<dyn Error>> {
    let mut req = reqwest::blocking::get(url)?;
    let mut h = Sha256::new();
    let mut r = tee::TeeReader::new(&mut req, &mut h);
    let mut w = BufWriter::new(fs::File::create(dst)?);
    io::copy(&mut r, &mut w)?;
    let dig = hex::encode(h.finalize().as_slice());
    println!("{}", dig);
    Ok(())
}

fn compute_hash<P: AsRef<Path>>(src: P) -> Result<(), Box<dyn Error>> {
    let mut h = Sha256::new();
    let mut f = fs::File::open(src)?;
    io::copy(&mut f, &mut h)?;

    println!("{:?}", h.finalize().as_slice());
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let data_dir = DataDir::at(&args.data_dir)?;
    data_dir.download("hurdat2.csv", &args.hurdat2_url)?;
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
