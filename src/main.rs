use cat5::{hurdat2, noaa, DataDir};
use clap::Parser;
use std::{error::Error, path::Path};

#[derive(Parser, Debug)]
struct Flags {
    #[clap(long, default_value_t=String::from("data"), help="directory where data will be stored")]
    data_dir: String,

    #[clap(long, default_value_t=noaa::hurdat2_url().to_owned(), help="NOAA URL to download hurdat2 data")]
    hurdat2_url: String,
}

impl Flags {
    fn data_dir(&self) -> &Path {
        Path::new(&self.data_dir)
    }

    fn hurdat2_url(&self) -> &str {
        &self.hurdat2_url
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let flags = Flags::parse();

    let data_dir = DataDir::at(flags.data_dir())?;

    let mut r = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(data_dir.download_and_open("hurdat2.csv", flags.hurdat2_url())?);

    while let Some(storm) = hurdat2::Storm::from_record_iter(&mut r.records()) {
        println!("{:?}", storm?);
    }

    Ok(())
}
