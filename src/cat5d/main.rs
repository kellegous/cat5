use cat5::hurdat2::{Status, Storm, StormIter};
use cat5::{map, noaa, DataDir};
use clap::{Args, Parser};
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use tiny_skia::ColorU8;

#[derive(Debug)]
struct ParseHexColorError;

impl fmt::Display for ParseHexColorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid hex color")
    }
}

impl Error for ParseHexColorError {}

#[derive(Copy, Clone, Debug)]
struct HexColor(ColorU8);

impl FromStr for HexColor {
    type Err = ParseHexColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("#") || s.len() != 7 {
            return Err(ParseHexColorError {});
        };
        let r = u8::from_str_radix(&s[1..3], 16).map_err(|_| ParseHexColorError {})?;
        let g = u8::from_str_radix(&s[3..5], 16).map_err(|_| ParseHexColorError {})?;
        let b = u8::from_str_radix(&s[5..7], 16).map_err(|_| ParseHexColorError {})?;
        Ok(HexColor(ColorU8::from_rgba(r, g, b, 0xff)))
    }
}

impl fmt::Display for HexColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = self.0;
        write!(f, "#{:02x}{:02x}{:02x}", c.red(), c.green(), c.blue())
    }
}
impl HexColor {
    fn from_rgb(r: u8, g: u8, b: u8) -> HexColor {
        HexColor(ColorU8::from_rgba(r, g, b, 0xff))
    }
    fn color(&self) -> ColorU8 {
        self.0
    }
}

#[derive(Parser, Debug)]
struct Flags {
    #[clap(
        long,
        default_value_t = String::from("data"),
        help = "directory where data should be written"
    )]
    data_dir: String,

    #[clap(
        long,
        default_value_t = noaa::hurdat2_url().to_owned(),
        help = "NOAA URL to download hurdat2 data"
    )]
    hurdat2_url: String,

    #[clap(flatten)]
    map: MapFlags,
}

#[derive(Args, Debug)]
struct MapFlags {
    #[clap(
        long = "map-svg-file",
        default_value_t = String::from("atlantic.svg"),
        help = "path to SVG map file"
    )]
    svg_file: String,

    #[clap(
        long = "map-land-color",
        default_value_t = HexColor::from_rgb(0xcb, 0xc0, 0xff),
        help = "hex color of land in map SVG"
    )]
    land_color: HexColor,
}

fn was_hurricane(storm: &Storm) -> bool {
    storm
        .track()
        .iter()
        .any(|e| e.status() == Status::Hurricane)
}

fn main() -> Result<(), Box<dyn Error>> {
    let flags = Flags::parse();

    let data_dir = DataDir::at(&flags.data_dir)?;
    let mut r = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(data_dir.download_and_open("hurdat2.csv", &flags.hurdat2_url)?);
    let hurricanes = StormIter::new(r.records())
        .filter(|r| match r {
            Ok(s) => was_hurricane(s),
            _ => true,
        })
        .collect::<Result<Vec<_>, _>>()?;
    println!("hurricanes: {}", hurricanes.len());

    let m = map::Map::build(&flags.map.svg_file, 10.0, &flags.map.land_color.color())?;
    println!("{:?}", m);

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
    println!("hurdat2_url = {}", &flags.hurdat2_url);

    Ok(())
}
