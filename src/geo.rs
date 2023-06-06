use std::error::Error;
use std::f64::consts::PI;
use std::fmt;
use std::str::FromStr;

const R: f64 = 6371e3;

#[derive(Debug, Clone)]
pub struct Location {
    lat: f64,
    lng: f64,
}

impl Location {
    pub fn new(lat: f64, lng: f64) -> Location {
        Location { lat, lng }
    }

    pub fn lat(&self) -> f64 {
        self.lat
    }

    pub fn lng(&self) -> f64 {
        self.lng
    }

    pub fn to_dms(&self) -> String {
        let (lat_d, lat_m, lat_s) = to_dms(self.lat);
        let (lng_d, lng_m, lng_s) = to_dms(self.lng);
        format!(
            "{:02}°{:02}′{:02}″{} {:03}°{:02}′{:02}″{}",
            lat_d,
            lat_m,
            lat_s,
            if self.lat < 0.0 { 'S' } else { 'N' },
            lng_d,
            lng_m,
            lng_s,
            if self.lng < 0.0 { 'W' } else { 'E' }
        )
    }

    pub fn distance_between(a: &Location, b: &Location) -> Distance {
        let φ1 = a.lat() * PI / 180.0;
        let φ2 = b.lat() * PI / 180.0;

        let δφ = (b.lat() - a.lat()) * PI / 180.0;
        let δλ = (b.lng() - a.lng()) * PI / 180.0;

        let a = (δφ / 2.0).sin() * (δφ / 2.0).sin()
            + φ1.cos() * φ2.cos() * (δλ / 2.0).sin() * (δλ / 2.0).sin();
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        Distance::from_meters(R * c)
    }

    pub fn destination_of(&self, bearing: Bearing, distance: Distance) -> Location {
        let φ1 = self.lat * PI / 180.0;
        let λ1 = self.lng * PI / 180.0;
        let θ = bearing.in_radians();
        let δ = distance.in_meters() / R;

        let sinφ2 = φ1.sin() * δ.cos() + φ1.cos() * δ.sin() * θ.cos();
        let φ2 = sinφ2.asin();
        let y = θ.sin() * δ.sin() * φ1.cos();
        let x = δ.cos() - φ1.sin() * φ2.sin();
        let λ2 = λ1 + y.atan2(x);

        Location {
            lat: φ2 * 180.0 / PI,
            lng: λ2 * 180.0 / PI,
        }
    }
}

impl FromStr for Location {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex::Regex::new(
            r#"(\d+)°(\d+)[′'](\d+)[″"]([NSns]) (\d+)°(\d+)[′'](\d+)[″"]([EWew])"#,
        )
        .unwrap();

        let caps = re.captures(s).ok_or(format!("invalid location: {}", s))?;

        let lat_d = caps.get(1).unwrap().as_str().parse::<i32>()?;
        let lat_m = caps.get(2).unwrap().as_str().parse::<i32>()?;
        let lat_s = caps.get(3).unwrap().as_str().parse::<i32>()?;
        let lat_v = lat_d as f64 + (lat_m as f64) / 60.0 + (lat_s as f64) / 3600.0;
        let lat_v = match caps.get(4).unwrap().as_str() {
            "N" | "n" => Ok(lat_v),
            "S" | "s" => Ok(-lat_v),
            _ => Err(format!("invalid location: {}", s)),
        }?;

        let lng_d = caps.get(5).unwrap().as_str().parse::<i32>()?;
        let lng_m = caps.get(6).unwrap().as_str().parse::<i32>()?;
        let lng_s = caps.get(7).unwrap().as_str().parse::<i32>()?;
        let lng_v = lng_d as f64 + (lng_m as f64) / 60.0 + (lng_s as f64) / 3600.0;
        let lng_v = match caps.get(8).unwrap().as_str() {
            "E" | "e" => Ok(lng_v),
            "W" | "w" => Ok(-lng_v),
            _ => Err(format!("invalid location: {}", s)),
        }?;

        Ok(Location {
            lat: lat_v,
            lng: lng_v,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Distance {
    m: f64,
}

impl Distance {
    pub fn from_nautical_miles(nm: f64) -> Distance {
        Distance { m: nm * 1852.0 }
    }

    pub fn from_meters(m: f64) -> Distance {
        Distance { m }
    }

    pub fn in_meters(&self) -> f64 {
        self.m
    }

    pub fn in_nautical_miles(&self) -> f64 {
        self.m / 1852.0
    }
}

impl std::fmt::Display for Distance {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} meters", self.m)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bearing {
    deg: f64,
}

impl Bearing {
    pub fn in_degrees(&self) -> f64 {
        self.deg
    }

    pub fn in_radians(&self) -> f64 {
        self.deg * PI / 180.0
    }

    pub fn from_degrees(deg: f64) -> Bearing {
        Bearing { deg }
    }

    pub fn from_radians(r: f64) -> Bearing {
        Bearing {
            deg: r * 180.0 / PI,
        }
    }

    pub fn north() -> Bearing {
        Bearing { deg: 0.0 }
    }

    pub fn east() -> Bearing {
        Bearing { deg: 90.0 }
    }

    pub fn south() -> Bearing {
        Bearing { deg: 180.0 }
    }

    pub fn west() -> Bearing {
        Bearing { deg: 270.0 }
    }

    pub fn to_dms(&self) -> String {
        let (d, m, s) = to_dms(self.deg);
        format!("{:03}°{:02}′{:02}″", d, m, s)
    }
}

impl FromStr for Bearing {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex::Regex::new(r#"(\d+)°(\d+)[′'](\d+)[″"]"#).unwrap();

        let caps = re.captures(s).ok_or(format!("invalid dms: {}", s))?;
        let d = caps.get(1).unwrap().as_str().parse::<i32>()?;
        let m = caps.get(2).unwrap().as_str().parse::<i32>()?;
        let s = caps.get(3).unwrap().as_str().parse::<i32>()?;

        Ok(Bearing {
            deg: d as f64 + m as f64 / 60.0 + s as f64 / 3600.0,
        })
    }
}

impl fmt::Display for Bearing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_dms())
    }
}

fn to_dms(v: f64) -> (i32, i32, i32) {
    let v = v.abs();

    let mut d = v as i32;

    let v = v - d as f64;

    let mut m = (v * 60.0) as i32;

    let v = v - m as f64 / 60.0;

    let mut s = (v * 3600.0).round() as i32;

    if s == 60 {
        s = 0;
        m += 1;
    }

    if m == 60 {
        m = 0;
        d += 1;
    }

    (d, m, s)
}

#[derive(Debug, Clone)]
pub struct Mercator {
    width: f64,
    height: f64,
    xoff: f64,
    yoff: f64,
}

impl Mercator {
    pub fn new(width: f64, height: f64, xoff: f64, yoff: f64) -> Mercator {
        Mercator {
            width,
            height,
            xoff,
            yoff,
        }
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn xoff(&self) -> f64 {
        self.xoff
    }

    pub fn yoff(&self) -> f64 {
        self.yoff
    }

    pub fn location_to_pt(&self, loc: &Location) -> (f64, f64) {
        let lat = loc.lat();
        let lng = loc.lng();
        let lat_rads = lat * (PI / 180.0);
        let merc_n = (PI / 4.0 + lat_rads / 2.0).tan().ln();
        (
            self.width / 2.0 + lng * self.width / 360.0 + self.xoff,
            self.height / 2.0 - (self.height * merc_n) / (2.0 * PI) + self.yoff,
        )
    }

    pub fn pt_to_location(&self, pt: (f64, f64)) -> Location {
        let (x, y) = pt;
        let a = (-2.0 * PI * (y - self.height / 2.0 - self.yoff)) / self.height;
        let lat_deg = 2.0 * a.exp().atan() - PI / 2.0;
        Location::new(
            lat_deg * (180.0 / PI),
            (360.0 / self.width) * (x - self.xoff - self.width / 2.0),
        )
    }
}

impl std::fmt::Display for Mercator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}, {}, {}, {}]",
            self.width, self.height, self.xoff, self.yoff
        )
    }
}

impl std::str::FromStr for Mercator {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vals: Vec<f64> = serde_json::from_str(s)?;
        if vals.len() != 4 {
            return Err(format!("invalid mercator: {}", s).into());
        }
        Ok(Mercator {
            width: vals[0],
            height: vals[1],
            xoff: vals[2],
            yoff: vals[3],
        })
    }
}
