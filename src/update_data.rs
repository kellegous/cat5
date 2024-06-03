use std::error::Error;

use csv_async::{AsyncReaderBuilder, StringRecordsStream};
use tiny_skia::ColorU8;
use tokio::io;

use crate::{
    geo::{self, Mercator},
    hurdat2::{Status, Storm},
    noaa, DataDir, FetchStrategy,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(long, default_value_t = noaa::hurdat2_url().to_owned())]
    hurdat2_url: String,

    #[clap(flatten)]
    map: ForMap,
}

#[derive(Debug, clap::Args)]
struct ForMap {
    #[clap(long = "map.svg-file", default_value_t=String::from("atlantic.svg"))]
    svg_file: String,

    #[clap(long = "map.land-color", value_parser=parse_color, default_value="#facbc0")]
    land_color: ColorU8,

    #[clap(long = "map.bin-size", default_value_t = 10.0)]
    bin_size: f64,

    #[clap(long = "map.projection", value_parser=parse_projection,)]
    projection: Mercator,
}

fn parse_color(s: &str) -> Result<ColorU8, String> {
    if !s.starts_with('#') || s.len() != 7 {
        return Err(format!("invalid color: {}", s));
    }

    let c = u32::from_str_radix(&s[1..], 16).map_err(|e| e.to_string())?;
    Ok(ColorU8::from_rgba(
        ((c >> 16) & 0xff) as u8,
        ((c >> 8) & 0xff) as u8,
        (c & 0xff) as u8,
        0,
    ))
}

fn parse_projection(s: &str) -> Result<geo::Mercator, String> {
    s.parse().map_err(|_| format!("invalid projection: {}", s))
}

fn default_projection() -> geo::Mercator {
    geo::Mercator::new(
        10368.61626248217,
        10310.9627199,
        -2160.1283880171186,
        -3566.7693291,
    )
}
async fn collect_storms<'a, R, F>(
    stream: &mut StringRecordsStream<'a, R>,
    filter: F,
) -> Result<Vec<Storm>, Box<dyn Error>>
where
    R: io::AsyncRead + Unpin + Send,
    F: Fn(&Storm) -> bool,
{
    let mut storms = vec![];
    while let Some(storm) = Storm::from_record_stream(stream).await {
        let storm = storm?;
        if filter(&storm) {
            storms.push(storm);
        }
    }
    Ok(storms)
}

pub async fn run(dir: &DataDir<'_>, args: Args) -> Result<(), Box<dyn Error>> {
    println!("{:?}", args);
    let f = dir
        .get_object("hurdat2.txt")
        .fetch(&args.hurdat2_url, FetchStrategy::Always)
        .await?
        .open()
        .await?;
    let mut stream = AsyncReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .create_reader(f);
    let mut records = stream.records();
    let storms = collect_storms(&mut records, |storm| {
        storm
            .track()
            .iter()
            .any(|e| e.status() == Status::Hurricane)
    })
    .await?;
    println!("{} hurricanes found", storms.len());

    Ok(())
}
