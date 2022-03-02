use super::{atcf, geo};
use chrono::{DateTime, Utc};
use std::error::Error;
use std::str::FromStr;

pub struct Storm {
	id: atcf::Id,
	name: Option<String>,
	track: Vec<TrackEntry>,
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
