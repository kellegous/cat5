use std::error::Error;
use std::fs;
use std::path::Path;
use tiny_skia::{ColorU8, Pixmap};

#[derive(Debug)]
pub struct Map {
	w: usize,
	h: usize,
}

impl Map {
	pub fn build<P: AsRef<Path>>(
		src: P,
		bin_size: f64,
		land_color: &ColorU8,
	) -> Result<Map, Box<dyn Error>> {
		let src = pixmap_from_svg(&src)?;

		let mut img = BitImage::from_pixmap(&src, bin_size, land_color);
		Ok(Map { w: 0, h: 0 })
	}
}

#[derive(Debug)]
struct BitImage {
	w: usize,
	h: usize,
	data: bit_vec::BitVec,
}

impl BitImage {
	fn new(w: usize, h: usize) -> BitImage {
		BitImage {
			w: w,
			h: h,
			data: bit_vec::BitVec::from_elem(w * h, false),
		}
	}

	fn width(&self) -> usize {
		self.w
	}

	fn height(&self) -> usize {
		self.h
	}

	fn get(&self, x: usize, y: usize) -> Option<bool> {
		self.data.get(y * self.w + x)
	}

	fn set(&mut self, x: usize, y: usize, v: bool) {
		self.data.set(y * self.w + x, v)
	}

	fn from_pixmap(src: &Pixmap, size: f64, land_color: &ColorU8) -> BitImage {
		let w = (src.width() as f64 / size) as usize;
		let h = (src.height() as f64 / size) as usize;
		let mut img = BitImage::new(w, h);
		for i in 0..w {
			let x = i as f64 * size;
			for j in 0..h {
				let y = j as f64 * size;
				let lt = src.pixel(x as u32, y as u32).unwrap().demultiply();
				let rt = src.pixel((x + size) as u32, y as u32).unwrap().demultiply();
				let rb = src
					.pixel((x + size) as u32, (y + size) as u32)
					.unwrap()
					.demultiply();
				let lb = src.pixel(x as u32, (y + size) as u32).unwrap().demultiply();
				if !is_similar_color(&lt, &land_color)
					&& !is_similar_color(&rt, &land_color)
					&& !is_similar_color(&rb, &land_color)
					&& !is_similar_color(&lb, &land_color)
				{
					continue;
				}

				img.set(i, j, true);
			}
		}
		img
	}
}

fn pixmap_from_svg<P: AsRef<Path>>(path: P) -> Result<tiny_skia::Pixmap, Box<dyn Error>> {
	let mut opts = usvg::Options::default();
	opts.fontdb.load_system_fonts();

	let data = fs::read(&path)?;
	let tree = usvg::Tree::from_data(&data, &opts)?;
	let size = tree.svg_node().size.to_screen_size();
	let mut pixels = match tiny_skia::Pixmap::new(size.width(), size.height()) {
		Some(p) => p,
		None => return Err("unable to create pixmap".into()),
	};
	if let None = resvg::render(&tree, usvg::FitTo::Original, pixels.as_mut()) {
		return Err("unable to render svg".into());
	}
	Ok(pixels)
}

fn u8_diff(a: u8, b: u8) -> u8 {
	if a > b {
		a - b
	} else {
		b - a
	}
}

fn is_similar_color(a: &ColorU8, b: &ColorU8) -> bool {
	u8_diff(a.red(), b.red()) < 8
		&& u8_diff(a.green(), b.green()) < 8
		&& u8_diff(a.blue(), b.blue()) < 8
}
