use super::map;
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
