use tiny_skia::{ColorU8, Pixmap};
use usvg::TreeParsing;

use crate::geo;
use std::{error::Error, fs, path::Path};

#[derive(Debug)]
pub struct Map {
    w: usize,
    h: usize,
    bin_size: f64,
    bins: Vec<Bin>,
    projection: geo::Mercator,
}

impl Map {
    pub fn width(&self) -> usize {
        self.w
    }

    pub fn height(&self) -> usize {
        self.h
    }

    pub fn bin_size(&self) -> f64 {
        self.bin_size
    }

    pub fn bins(&self) -> &[Bin] {
        &self.bins
    }

    pub fn projection(&self) -> &geo::Mercator {
        &self.projection
    }

    pub fn build<P: AsRef<Path>>(
        src: P,
        bin_size: f64,
        land_color: ColorU8,
        projection: geo::Mercator,
    ) -> Result<Self, Box<dyn Error>> {
        let src = pixmap_from_svg(src)?;

        let img = BitImage::from_pixmap(&src, bin_size, land_color);

        let bins = (0..img.w)
            .flat_map(|i| (0..img.h).map(move |j| (i, j)))
            .filter(|(i, j)| img.get(*i, *j).unwrap())
            .map(|(i, j)| Bin {
                i: i as i32,
                j: j as i32,
            })
            .collect::<Vec<_>>();

        Ok(Map {
            w: img.w,
            h: img.h,
            bin_size,
            bins,
            projection,
        })
    }
}
#[derive(Debug)]
pub struct Bin {
    pub i: i32,
    pub j: i32,
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

#[derive(Debug)]
struct BitImage {
    w: usize,
    h: usize,
    data: bit_vec::BitVec,
}

impl BitImage {
    fn new(w: usize, h: usize) -> BitImage {
        BitImage {
            w,
            h,
            data: bit_vec::BitVec::from_elem(w * h, false),
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<bool> {
        self.data.get(y * self.w + x)
    }

    fn set(&mut self, x: usize, y: usize, v: bool) {
        self.data.set(y * self.w + x, v);
    }

    fn from_pixmap(src: &Pixmap, size: f64, land_color: ColorU8) -> BitImage {
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

fn pixmap_from_svg<P: AsRef<Path>>(src: P) -> Result<Pixmap, Box<dyn Error>> {
    let opts = usvg::Options::default();
    let data = fs::read(src)?;
    let tree = usvg::Tree::from_data(&data, &opts)?;
    let tree = resvg::Tree::from_usvg(&tree);
    let size = tree.size;
    let mut pixels =
        Pixmap::new(size.width() as u32, size.height() as u32).ok_or("unable to create pixmap")?;
    tree.render(tiny_skia::Transform::default(), &mut pixels.as_mut());
    Ok(pixels)
}
