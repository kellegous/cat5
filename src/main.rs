use cat5::{
    debug, geo,
    hurdat2::{self, Status, Storm},
    noaa, DataDir, Map,
};
use clap::{Args, Parser};
use std::{error::Error, path::Path};

#[derive(Parser, Debug)]
struct Flags {
    #[clap(long, default_value_t=String::from("data"), help="directory where data will be stored")]
    data_dir: String,

    #[clap(long, default_value_t=noaa::hurdat2_url().to_owned(), help="NOAA URL to download hurdat2 data")]
    hurdat2_url: String,

    #[clap(flatten)]
    map: MapFlags,

    #[clap(flatten)]
    web: WebFlags,
}

impl Flags {
    fn data_dir(&self) -> &Path {
        Path::new(&self.data_dir)
    }

    fn hurdat2_url(&self) -> &str {
        &self.hurdat2_url
    }
}

#[derive(Args, Debug, Clone)]
struct MapFlags {
    #[clap(
        long = "map.svg-file",
        default_value_t=String::from("atlantic.svg"),
        help="SVG file to use as map")]
    svg_file: String,

    #[clap(
        long = "map.land-color",
        default_value_t=Color::from_rgb(0xfa, 0xcb, 0xc0 ),
        value_parser=Color::from_arg,
        help="color of land in SVG map")]
    land_color: Color,

    #[clap(
        long = "map.bin-size",
        default_value_t = 10.0,
        help = "size of map bins"
    )]
    bin_size: f64,

    #[clap(
        long = "map.mercator",
        default_value_t=default_mercator(),
        value_parser=mercator_from_arg,
        help="mercator projection")]
    mercator: geo::Mercator,
}

impl MapFlags {
    fn land_color(&self) -> tiny_skia::ColorU8 {
        self.land_color.0
    }
}

#[derive(Args, Debug)]
struct WebFlags {
    #[clap(long = "web.addr", default_value_t=String::from("0.0.0.0:8080"), help="address to listen on")]
    addr: String,
}

fn default_mercator() -> geo::Mercator {
    geo::Mercator::new(
        10368.61626248217,
        10310.9627199,
        -2160.1283880171186,
        -3566.7693291,
    )
}

fn mercator_from_arg(s: &str) -> Result<geo::Mercator, String> {
    s.parse::<geo::Mercator>()
        .map_err(|_| format!("invalid mercator: {}", s))
}

#[derive(Debug, Clone)]
struct Color(tiny_skia::ColorU8);

impl Color {
    fn from_arg(s: &str) -> Result<Color, String> {
        s.parse::<Color>()
            .map_err(|_| format!("invalid color: {}", s))
    }

    fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color(tiny_skia::ColorU8::from_rgba(r, g, b, 0xff))
    }
}
impl std::str::FromStr for Color {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('#') || s.len() != 7 {
            return Err(format!("invalid color: {}", s).into());
        }
        Ok(Color(tiny_skia::ColorU8::from_rgba(
            u8::from_str_radix(&s[1..3], 16)?,
            u8::from_str_radix(&s[3..5], 16)?,
            u8::from_str_radix(&s[5..7], 16)?,
            0xff,
        )))
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let c = self.0;
        write!(f, "#{:02x}{:02x}{:02x}", c.red(), c.green(), c.blue())
    }
}

fn collect_storms<R, F>(
    iter: &mut csv::StringRecordsIter<R>,
    f: F,
) -> Result<Vec<Storm>, Box<dyn Error>>
where
    R: std::io::Read,
    F: Fn(&hurdat2::Storm) -> bool,
{
    let mut storms = Vec::new();
    while let Some(storm) = hurdat2::Storm::from_record_iter(iter) {
        let storm = storm?;
        if f(&storm) {
            storms.push(storm);
        }
    }
    Ok(storms)
}

fn main() -> Result<(), Box<dyn Error>> {
    let flags = Flags::parse();

    let data_dir = DataDir::at(flags.data_dir())?;

    let mut r = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(data_dir.download_and_open("hurdat2.csv", flags.hurdat2_url())?);
    let hurricanes = collect_storms(&mut r.records(), |s| {
        s.track().iter().any(|e| e.status() == Status::Hurricane)
    })?;

    println!("hurricanes: {}", hurricanes.len());

    let map = Map::build(
        &flags.map.svg_file,
        flags.map.bin_size,
        flags.map.land_color(),
        flags.map.mercator.clone(),
        7,
    )?;
    debug::render_map(data_dir.join("map.pdf"), &map)?;

    Ok(())
}
