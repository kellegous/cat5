use cat5::noaa;
use clap::{Args, Parser};
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use tiny_skia::ColorU8;

#[derive(Parser, Debug)]
pub struct Flags {
	#[clap(
        long,
        default_value_t = String::from("data"),
        help = "directory where data should be written"
    )]
	data_dir: String,

	#[clap(
		long,
		default_value_t = String::from("dist"),
		help = "directory where web assets are kept"
	)]
	dist_dir: String,

	#[clap(
        long,
        default_value_t = noaa::hurdat2_url().to_owned(),
        help = "NOAA URL to download hurdat2 data"
    )]
	hurdat2_url: String,

	#[clap(flatten)]
	map: MapFlags,

	#[clap(flatten)]
	web: WebFlags,
}

impl Flags {
	pub fn data_dir(&self) -> &Path {
		Path::new(&self.data_dir)
	}

	pub fn dist_dir(&self) -> &Path {
		Path::new(&self.dist_dir)
	}

	pub fn hurdat2_url(&self) -> &str {
		&self.hurdat2_url
	}

	pub fn for_map(&self) -> &MapFlags {
		&self.map
	}

	pub fn for_web(&self) -> &WebFlags {
		&self.web
	}
}

#[derive(Args, Debug)]
pub struct MapFlags {
	#[clap(
        long = "map-svg-file",
        default_value_t = String::from("atlantic.svg"),
        help = "path to SVG map file"
    )]
	svg_file: String,

	#[clap(
        long = "map-land-color",
        default_value_t = HexColor::from_rgb(0xfa, 0xcb, 0xc0),
        help = "hex color of land in map SVG"
    )]
	land_color: HexColor,
}

impl MapFlags {
	pub fn svg_file(&self) -> &Path {
		Path::new(&self.svg_file)
	}

	pub fn land_color(&self) -> &ColorU8 {
		&self.land_color.0
	}
}

#[derive(Args, Debug)]
pub struct WebFlags {
	#[clap(
        long = "web-bind-addr",
        default_value_t = String::from("127.0.0.1:8080"),
        help = "the address to bind"
    )]
	bind_addr: String,

	#[clap(long = "web-rebuild-assets", help = "automatically rebuild web assets")]
	rebuild_assets: bool,
}

impl WebFlags {
	pub fn bind_addr(&self) -> &str {
		&self.bind_addr
	}

	pub fn should_rebuild_assets(&self) -> bool {
		self.rebuild_assets
	}
}

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

impl HexColor {
	fn from_rgb(r: u8, g: u8, b: u8) -> HexColor {
		HexColor(ColorU8::from_rgba(r, g, b, 0xff))
	}
}

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
