use std::error::Error;
use std::fs;
use std::path::Path;
use tiny_skia::{ColorU8, Pixmap};

#[derive(Debug)]
pub struct Map {
	w: usize,
	h: usize,
	bin_size: f64,
	bins: Vec<Bin>,
}

impl Map {
	pub fn build<P: AsRef<Path>>(
		src: P,
		bin_size: f64,
		land_color: &ColorU8,
	) -> Result<Map, Box<dyn Error>> {
		let src = pixmap_from_svg(&src)?;

		let mut img = BitImage::from_pixmap(&src, bin_size, land_color);

		simplify::with_flood_fill(&mut img, 5);

		let mut bins = Vec::new();
		for i in 0..img.width() {
			for j in 0..img.height() {
				if img.get(i, j).unwrap() {
					bins.push(Bin {
						i: i as i32,
						j: j as i32,
					})
				}
			}
		}

		Ok(Map {
			w: img.width(),
			h: img.height(),
			bin_size,
			bins,
		})
	}

	pub fn bins(&self) -> &[Bin] {
		&self.bins
	}

	pub fn bin_size(&self) -> f64 {
		self.bin_size
	}

	pub fn width(&self) -> usize {
		self.w
	}

	pub fn height(&self) -> usize {
		self.h
	}
}

#[derive(Debug)]
pub struct Bin {
	pub i: i32,
	pub j: i32,
}

#[derive(Debug)]
pub struct BitImage {
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
				if is_similar_color(&lt, &land_color)
					|| is_similar_color(&rt, &land_color)
					|| is_similar_color(&rb, &land_color)
					|| is_similar_color(&lb, &land_color)
				{
					img.set(i, j, true);
				}
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

mod simplify {
	use super::BitImage;

	struct WithFloodFill<'a> {
		src: &'a mut BitImage,
		data: Vec<u8>,
		groups: Vec<u32>,
	}

	impl<'a> WithFloodFill<'a> {
		fn get(&self, x: usize, y: usize) -> u8 {
			let w = self.src.width();
			self.data[y * w + x]
		}

		fn set(&mut self, x: usize, y: usize, v: u8) {
			let w = self.src.width();
			self.data[y * w + x] = v;
		}

		fn is_inside(&self, x: i32, y: i32) -> bool {
			if x < 0 || y < 0 {
				return false;
			}
			let x = x as usize;
			let y = y as usize;
			if x >= self.src.width() || y >= self.src.height() {
				return false;
			}
			if self.get(x, y) != 0 || !self.src.get(x, y).unwrap() {
				return false;
			}
			true
		}

		fn scan(&self, s: &mut Vec<(i32, i32)>, lx: i32, rx: i32, y: i32) {
			let mut added = false;
			for x in lx..=rx {
				if !self.is_inside(x, y) {
					added = false
				} else if !added {
					s.push((x, y));
					added = true
				}
			}
		}

		fn fill(&mut self, x: usize, y: usize) {
			let x = x as i32;
			let y = y as i32;
			if !self.is_inside(x, y) {
				return;
			}
			let id = self.groups.len() as u8 + 1;
			let mut n = 0;

			let mut s = vec![(x, y)];
			while let Some((mut x, y)) = s.pop() {
				let mut lx = x;
				while self.is_inside(lx - 1, y) {
					self.set(lx as usize - 1, y as usize, id);
					n += 1;
					lx -= 1;
				}
				while self.is_inside(x, y) {
					self.set(x as usize, y as usize, id);
					n += 1;
					x += 1;
				}
				self.scan(&mut s, lx, x - 1, y + 1);
				self.scan(&mut s, lx, x - 1, y - 1);
			}
			self.groups.push(n);
		}
		fn simplify(src: &'a mut BitImage, lim: u32) {
			let w = src.width();
			let h = src.height();
			let mut state = WithFloodFill {
				src: src,
				data: vec![0; w * h],
				groups: Vec::new(),
			};
			for i in 0..w {
				for j in 0..h {
					state.fill(i, j);
				}
			}
			for i in 0..w {
				for j in 0..h {
					let id = state.get(i, j);
					if id == 0 {
						continue;
					}
					if state.groups[id as usize - 1] < lim {
						state.src.set(i, j, false);
					}
				}
			}
		}
	}

	pub fn with_flood_fill(img: &mut BitImage, lim: u32) {
		WithFloodFill::simplify(img, lim)
	}
}
