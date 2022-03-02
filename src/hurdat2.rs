use super::{atcf, geo};
use chrono::prelude::*;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use std::error::Error;
use std::io;
use std::str::FromStr;

pub struct Storm {
	id: atcf::Id,
	name: Option<String>,
	track: Vec<TrackEntry>,
}

impl Storm {
	fn next_from_iterator<R>(
		rec: csv::Result<csv::StringRecord>,
		iter: &mut csv::StringRecordsIter<R>,
	) -> Result<Storm, Box<dyn Error>>
	where
		R: io::Read,
	{
		let header = Header::from_record(&rec?)?;
		let track_entries = iter
			.take(header.num_track_entries as usize)
			.map(|rec| TrackEntry::from_record(&rec?))
			.collect::<Result<Vec<_>, Box<dyn Error>>>()?;
		if track_entries.len() == header.num_track_entries {
			Ok(Storm {
				id: header.id,
				name: header.name,
				track: track_entries,
			})
		} else {
			Err(format!(
				"expected {} track entries but only read {}",
				header.num_track_entries,
				track_entries.len()
			)
			.into())
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Indicator {
	// ClosetApproach indicates closest approach to coast, not followed by a landfall
	ClosestApproach,

	// Genesis indicates Genesis
	Genesis,

	// IntensityPeak indicates an intensity peak in terms of both pressure and wind
	IntensityPeak,

	// Landfall indicates landfall (center of system crossing a coastline)
	Landfall,

	// MinCentralPressure indicates minimum central pressure
	MinCentralPressure,

	// RapidChanges indicates additional detail on the intensity of the cycle when rapid changes are underway
	RapidChanges,

	// StatusChange indicates a change in the status of the system
	StatusChange,

	// Track provides additional detail on the track (position) of the cyclone
	Track,

	// MaxWind indicates maximum sustained wind speed
	MaxWind,
}

impl Indicator {
	pub fn to_str(&self) -> char {
		match self {
			Indicator::ClosestApproach => 'C',
			Indicator::Genesis => 'G',
			Indicator::IntensityPeak => 'I',
			Indicator::Landfall => 'L',
			Indicator::MinCentralPressure => 'P',
			Indicator::RapidChanges => 'R',
			Indicator::StatusChange => 'S',
			Indicator::Track => 'T',
			Indicator::MaxWind => 'W',
		}
	}

	pub fn from_char(c: char) -> Result<Indicator, Box<dyn Error>> {
		match c {
			'C' => Ok(Indicator::ClosestApproach),
			'G' => Ok(Indicator::Genesis),
			'I' => Ok(Indicator::IntensityPeak),
			'L' => Ok(Indicator::Landfall),
			'P' => Ok(Indicator::MinCentralPressure),
			'R' => Ok(Indicator::RapidChanges),
			'S' => Ok(Indicator::StatusChange),
			'T' => Ok(Indicator::Track),
			'W' => Ok(Indicator::MaxWind),
			_ => Err(format!("invalid indicator: {}", c).into()),
		}
	}
}

impl std::fmt::Display for Indicator {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.to_str())
	}
}

impl FromStr for Indicator {
	type Err = Box<dyn Error>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.len() == 1 {
			Indicator::from_char(s.chars().nth(0).unwrap())
		} else {
			Err(format!("invalid indicator: {}", s).into())
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub enum Status {
	TropicalDepression,
	TropicalStorm,
	Hurricane,
	Extratropical,
	SubtropicalDepression,
	SubtropicalStorm,
	Low,
	TropicalWave,
	Disturbance,
}

impl Status {
	pub fn to_str(&self) -> &str {
		match self {
			Status::TropicalDepression => "TD",
			Status::TropicalStorm => "TS",
			Status::Hurricane => "HU",
			Status::Extratropical => "EX",
			Status::SubtropicalDepression => "SD",
			Status::SubtropicalStorm => "SS",
			Status::Low => "LO",
			Status::TropicalWave => "WV",
			Status::Disturbance => "DB",
		}
	}
}

impl FromStr for Status {
	type Err = Box<dyn Error>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"TD" => Ok(Status::TropicalDepression),
			"TS" => Ok(Status::TropicalStorm),
			"HU" => Ok(Status::Hurricane),
			"EX" => Ok(Status::Extratropical),
			"SD" => Ok(Status::SubtropicalDepression),
			"SS" => Ok(Status::SubtropicalStorm),
			"LO" => Ok(Status::Low),
			"WV" => Ok(Status::TropicalWave),
			"DB" => Ok(Status::Disturbance),
			_ => Err(format!("invalid status: {}", s).into()),
		}
	}
}

#[derive(Debug)]
pub struct TrackEntry {
	time: DateTime<Utc>,
	indicator: Option<Indicator>,
	status: Status,
	location: geo::Location,
	max_sustained_wind: i32,
	min_pressure: i32,
	wind_radii_34kts: WindRadii,
	wind_radii_50kts: WindRadii,
	wind_radii_64kts: WindRadii,
}

impl TrackEntry {
	pub fn time(&self) -> &DateTime<Utc> {
		&self.time
	}

	pub fn indicator(&self) -> Option<Indicator> {
		self.indicator
	}

	pub fn status(&self) -> Status {
		self.status
	}

	pub fn location(&self) -> &geo::Location {
		&self.location
	}

	pub fn wind_radii_34kts(&self) -> &WindRadii {
		&self.wind_radii_34kts
	}

	pub fn wind_radii_50kts(&self) -> &WindRadii {
		&self.wind_radii_50kts
	}

	pub fn wind_radii_64kts(&self) -> &WindRadii {
		&self.wind_radii_64kts
	}

	pub fn max_sustained_wind(&self) -> i32 {
		self.max_sustained_wind
	}

	pub fn min_pressure(&self) -> i32 {
		self.min_pressure
	}

	pub fn from_record(record: &csv::StringRecord) -> Result<TrackEntry, Box<dyn Error>> {
		let d = NaiveDate::parse_from_str(record.get(0).unwrap().trim(), "%Y%m%d")?;
		let t = NaiveTime::parse_from_str(record.get(1).unwrap().trim(), "%H%M")?;
		let time = Utc.from_utc_datetime(&NaiveDateTime::new(d, t));

		Ok(TrackEntry {
			time,
			indicator: match record.get(2).unwrap().trim() {
				"" => None,
				v => Some(v.parse::<Indicator>()?),
			},
			status: record.get(3).unwrap().trim().parse::<Status>()?,
			location: parse_location(record.get(4).unwrap().trim(), record.get(5).unwrap().trim())?,
			max_sustained_wind: record
				.get(6)
				.unwrap()
				.trim()
				.parse::<i32>()
				.map_err(|_| "invalid max_sustained_wind")?,
			min_pressure: record
				.get(7)
				.unwrap()
				.trim()
				.parse::<i32>()
				.map_err(|_| "invalid min_pressure")?,
			wind_radii_34kts: WindRadii::from_strs(
				record.get(8).unwrap().trim(),
				record.get(9).unwrap().trim(),
				record.get(10).unwrap().trim(),
				record.get(11).unwrap().trim(),
			)?,
			wind_radii_50kts: WindRadii::from_strs(
				record.get(12).unwrap().trim(),
				record.get(13).unwrap().trim(),
				record.get(14).unwrap().trim(),
				record.get(15).unwrap().trim(),
			)?,
			wind_radii_64kts: WindRadii::from_strs(
				record.get(16).unwrap().trim(),
				record.get(17).unwrap().trim(),
				record.get(18).unwrap().trim(),
				record.get(19).unwrap().trim(),
			)?,
		})
	}
}

#[derive(Debug)]
pub struct WindRadii {
	ne: i32,
	se: i32,
	sw: i32,
	nw: i32,
}

impl WindRadii {
	fn from_strs(ne: &str, se: &str, sw: &str, nw: &str) -> Result<WindRadii, Box<dyn Error>> {
		let ne = ne
			.parse::<i32>()
			.map_err(|_| format!("invalid ne: {}", ne))?;
		let se = se
			.parse::<i32>()
			.map_err(|_| format!("invalid se: {}", se))?;
		let sw = sw
			.parse::<i32>()
			.map_err(|_| format!("invalid sw: {}", sw))?;
		let nw = nw
			.parse::<i32>()
			.map_err(|_| format!("invalid nw: {}", nw))?;
		Ok(WindRadii { ne, se, sw, nw })
	}

	pub fn max_radius(&self) -> Option<geo::Distance> {
		let r = std::cmp::max(
			std::cmp::max(self.se, self.ne),
			std::cmp::max(self.sw, self.nw),
		);
		if r == -999 {
			None
		} else {
			Some(geo::Distance::from_nautical_miles(r as f64))
		}
	}
}

struct Header {
	id: atcf::Id,
	name: Option<String>,
	num_track_entries: usize,
}

impl Header {
	fn from_record(rec: &csv::StringRecord) -> Result<Header, Box<dyn Error>> {
		if rec.len() == 4 {
			let id = rec.get(0).unwrap().trim().parse::<atcf::Id>()?;
			let name = match rec.get(1).unwrap().trim() {
				"UNNAMED" => None,
				v => Some(v.to_owned()),
			};
			let num_track_entries = rec.get(2).unwrap().parse::<usize>()?;
			Ok(Header {
				id,
				name,
				num_track_entries,
			})
		} else {
			Err(format!("header has {} columns instead of 4", rec.len()).into())
		}
	}
}

pub struct StormIter<'a, R: 'a> {
	iter: csv::StringRecordsIter<'a, R>,
}

impl<'a, R: io::Read> Iterator for StormIter<'a, R> {
	type Item = Result<Storm, Box<dyn Error>>;

	fn next(&mut self) -> Option<Self::Item> {
		self.iter
			.next()
			.and_then(|rec| Some(Storm::next_from_iterator(rec, &mut self.iter)))
	}
}

fn parse_location(lat: &str, lng: &str) -> Result<geo::Location, Box<dyn Error>> {
	let lat = match lat {
		"" => return Err("empty lat".into()),
		_ => {
			let v = &lat[0..lat.len() - 1].parse::<f64>()?;
			match lat.chars().last().unwrap() {
				'E' | 'e' => *v,
				'W' | 'w' => -v,
				_ => return Err(format!("invalid lat: {}", lat).into()),
			}
		}
	};
	let lng = match lng {
		"" => return Err("empty lng".into()),
		_ => {
			let v = &lng[0..lng.len() - 1].parse::<f64>()?;
			match lng.chars().last().unwrap() {
				'N' | 'n' => *v,
				'S' | 's' => -v,
				_ => return Err(format!("invalid lng: {}", lng).into()),
			}
		}
	};

	Ok(geo::Location::new(lat, lng))
}
