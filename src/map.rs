use tiny_skia::{ColorU8, Pixmap};

use crate::geo;
use std::{error::Error, path::Path};
use tokio::fs;

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

    pub async fn build<P: AsRef<Path>>(
        src: P,
        bin_size: f64,
        land_color: ColorU8,
        projection: geo::Mercator,
        flood_limit: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let src = pixmap_from_svg(src).await?;

        let mut img = BitImage::from_pixmap(&src, bin_size, land_color);

        simplify::with_flood_fill(&mut img, flood_limit);

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
pub struct BitImage {
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

async fn pixmap_from_svg<P: AsRef<Path>>(src: P) -> Result<Pixmap, Box<dyn Error>> {
    let opts = usvg::Options::default();
    let data = fs::read(src).await?;
    let tree = usvg::Tree::from_data(&data, &opts)?;
    let size = tree.size();
    let mut pixels =
        Pixmap::new(size.width() as u32, size.height() as u32).ok_or("unable to create pixmap")?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixels.as_mut());
    Ok(pixels)
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
                src,
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
