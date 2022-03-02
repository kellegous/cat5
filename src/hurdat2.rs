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
}
