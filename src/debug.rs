use super::{geo, hurdat2, map};
use cairo::{Context, PdfSurface};
use std::error::Error;
use std::path::Path;

struct Color {
	r: u8,
	g: u8,
	b: u8,
}

impl Color {
	fn from_rgb(r: u8, g: u8, b: u8) -> Color {
		Color { r, g, b }
	}

	fn from_u32(c: u32) -> Color {
		Self::from_rgb(
			(c >> 16 & 0xff) as u8,
			(c >> 8 & 0xff) as u8,
			(c & 0xff) as u8,
		)
	}

	fn set(&self, ctx: &Context) {
		let r = self.r as f64 / 255.0;
		let g = self.g as f64 / 255.0;
		let b = self.b as f64 / 255.0;
		ctx.set_source_rgb(r, g, b);
	}

	fn set_with_alpha(&self, ctx: &Context, a: f64) {
		let r = self.r as f64 / 255.0;
		let g = self.g as f64 / 255.0;
		let b = self.b as f64 / 255.0;
		ctx.set_source_rgba(r, g, b, a);
	}
}

pub fn render_map<P: AsRef<Path>>(dst: P, m: &map::Map) -> Result<(), Box<dyn Error>> {
	let surface = PdfSurface::new(
		m.width() as f64 * m.bin_size(),
		m.height() as f64 * m.bin_size(),
		&dst,
	)?;
	let ctx = Context::new(&surface)?;

	ctx.new_path();
	for bin in m.bins() {
		let x = bin.i as f64 * m.bin_size();
		let y = bin.j as f64 * m.bin_size();
		Color::from_u32(0x0099ff).set(&ctx);
		ctx.rectangle(x + 0.5, y + 0.5, m.bin_size() - 1.0, m.bin_size() - 1.0);
	}
	ctx.fill()?;

	surface.finish();
	Ok(())
}

pub mod export {
	use super::{geo, hurdat2, map};
	use serde::ser::{self, SerializeMap, SerializeSeq};
	use std::error::Error;
	use std::io;

	struct Location<'a> {
		loc: &'a geo::Location,
	}

	impl<'a> Location<'a> {
		fn new(loc: &geo::Location) -> Location {
			Location { loc }
		}
	}

	impl<'a> ser::Serialize for Location<'a> {
		fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
		where
			S: ser::Serializer,
		{
			let mut s = s.serialize_seq(Some(2))?;
			s.serialize_element(&self.loc.lat())?;
			s.serialize_element(&self.loc.lng())?;
			s.end()
		}
	}

	struct TrackEntry<'a> {
		entry: &'a hurdat2::TrackEntry,
	}

	impl<'a> TrackEntry<'a> {
		fn new(entry: &hurdat2::TrackEntry) -> TrackEntry {
			TrackEntry { entry }
		}
	}

	impl<'a> ser::Serialize for TrackEntry<'a> {
		fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
		where
			S: ser::Serializer,
		{
			let mut s = s.serialize_map(Some(8))?;
			s.serialize_entry("time", &self.entry.time().timestamp_millis())?;
			s.serialize_entry("status", self.entry.status().to_str())?;
			s.serialize_entry("location", &Location::new(self.entry.location()))?;
			s.serialize_entry("max_wind", &self.entry.max_sustained_wind())?;
			s.serialize_entry("min_pressure", &self.entry.min_pressure())?;
			let r34 = self
				.entry
				.wind_radii_34kts()
				.max_radius()
				.map(|d| d.in_meters());
			s.serialize_entry("wind_radius_34kt", &r34)?;
			let r50 = self
				.entry
				.wind_radii_50kts()
				.max_radius()
				.map(|d| d.in_meters());
			s.serialize_entry("wind_radius_50kt", &r50)?;
			let r64 = self
				.entry
				.wind_radii_64kts()
				.max_radius()
				.map(|d| d.in_meters());
			s.serialize_entry("wind_radius_64kt", &r64)?;
			s.end()
		}
	}

	struct Storm<'a> {
		storm: &'a hurdat2::Storm,
	}

	impl<'a> Storm<'a> {
		fn new(storm: &hurdat2::Storm) -> Storm {
			Storm { storm }
		}
	}

	impl<'a> ser::Serialize for Storm<'a> {
		fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
		where
			S: ser::Serializer,
		{
			let mut s = s.serialize_map(Some(3))?;
			s.serialize_entry("id", &format!("{}", self.storm.id()))?;
			s.serialize_entry("name", &self.storm.name())?;
			let track = self
				.storm
				.track()
				.iter()
				.map(|e| TrackEntry::new(e))
				.collect::<Vec<_>>();
			s.serialize_entry("track", &track)?;
			s.end()
		}
	}

	struct Map<'a> {
		m: &'a map::Map,
	}

	impl<'a> Map<'a> {
		fn new(m: &map::Map) -> Map {
			Map { m }
		}
	}

	impl<'a> ser::Serialize for Map<'a> {
		fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
		where
			S: ser::Serializer,
		{
			let mut s = s.serialize_map(Some(5))?;
			s.serialize_entry("width", &self.m.width())?;
			s.serialize_entry("height", &self.m.height())?;
			s.serialize_entry("bin_size", &self.m.bin_size())?;
			let bins = self
				.m
				.bins()
				.iter()
				.map(|b| Bin::new(b))
				.collect::<Vec<_>>();
			s.serialize_entry("bins", &bins)?;
			s.serialize_entry("mercator", &Mercator::new(self.m.mercator()))?;
			s.end()
		}
	}

	struct Bin<'a> {
		bin: &'a map::Bin,
	}

	impl<'a> Bin<'a> {
		fn new(bin: &map::Bin) -> Bin {
			Bin { bin }
		}
	}

	impl<'a> ser::Serialize for Bin<'a> {
		fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
		where
			S: ser::Serializer,
		{
			let mut s = s.serialize_seq(Some(2))?;
			s.serialize_element(&self.bin.i)?;
			s.serialize_element(&self.bin.j)?;
			s.end()
		}
	}

	struct Mercator<'a> {
		m: &'a geo::Mercator,
	}

	impl<'a> Mercator<'a> {
		fn new(m: &geo::Mercator) -> Mercator {
			Mercator { m }
		}
	}

	impl<'a> ser::Serialize for Mercator<'a> {
		fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
		where
			S: ser::Serializer,
		{
			let mut s = s.serialize_map(Some(4))?;
			s.serialize_entry("width", &self.m.width())?;
			s.serialize_entry("height", &self.m.height())?;
			s.serialize_entry("xoff", &self.m.xoff())?;
			s.serialize_entry("yoff", &self.m.yoff())?;
			s.end()
		}
	}

	pub fn storms<W: io::Write>(w: W, storms: &[hurdat2::Storm]) -> Result<(), Box<dyn Error>> {
		let storms = storms.iter().map(|s| Storm::new(s)).collect::<Vec<_>>();
		serde_json::to_writer(w, &storms)?;
		Ok(())
	}

	pub fn map<W: io::Write>(w: W, m: &map::Map) -> Result<(), Box<dyn Error>> {
		serde_json::to_writer(w, &Map::new(m))?;
		Ok(())
	}
}
