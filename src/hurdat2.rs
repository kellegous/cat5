use std::error::Error;
use std::io;
use std::str::FromStr;

use super::{atcf, geo};
use chrono::prelude::*;
use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug)]
pub struct Storm {
    id: atcf::Id,
    name: Option<String>,
    track: Vec<TrackEntry>,
}

impl Storm {
    pub fn track(&self) -> &[TrackEntry] {
        &self.track
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn id(&self) -> &atcf::Id {
        &self.id
    }

    pub fn from_record_iter<R>(
        iter: &mut csv::StringRecordsIter<R>,
    ) -> Option<Result<Storm, Box<dyn Error>>>
    where
        R: io::Read,
    {
        iter.next().map(|r| Storm::from_record(r, iter))
    }

    fn from_record<R>(
        record: csv::Result<csv::StringRecord>,
        iter: &mut csv::StringRecordsIter<R>,
    ) -> Result<Storm, Box<dyn Error>>
    where
        R: io::Read,
    {
        let header = Header::from_record(&record?)?;
        let track_entries = iter
            .take(header.num_track_entries)
            .map(|r| TrackEntry::from_record(&r?))
            .collect::<Result<Vec<_>, _>>()?;
        if track_entries.len() != header.num_track_entries {
            return Err(format!(
                "expected {} track entries but got {}",
                header.num_track_entries,
                track_entries.len()
            )
            .into());
        }
        Ok(Storm {
            id: header.id,
            name: header.name,
            track: track_entries,
        })
    }
}

fn parse_location(lat: &str, lng: &str) -> Result<geo::Location, Box<dyn Error>> {
    if lat.is_empty() {
        return Err("empty latitude".into());
    }

    if lng.is_empty() {
        return Err("empty longitude".into());
    }

    let v = lat[..lat.len() - 1].parse::<f64>()?;
    let lat = match lat.chars().last().unwrap() {
        'N' | 'n' => v,
        'S' | 's' => -v,
        _ => return Err(format!("invalid latitude: {}", lat).into()),
    };

    let v = lng[..lng.len() - 1].parse::<f64>()?;
    let lng = match lng.chars().last().unwrap() {
        'E' | 'e' => v,
        'W' | 'w' => -v,
        _ => return Err(format!("invalid longitude: {}", lng).into()),
    };

    Ok(geo::Location::new(lat, lng))
}

fn parse_wind_radii(
    record: &csv::StringRecord,
    offset: usize,
) -> Result<WindRadii, Box<dyn Error>> {
    WindRadii::from_strs(
        record.get(offset).ok_or("missing ne")?.trim(),
        record.get(offset + 1).ok_or("missing se")?.trim(),
        record.get(offset + 2).ok_or("missing sw")?.trim(),
        record.get(offset + 3).ok_or("missing nw")?.trim(),
    )
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
    pub fn time(&self) -> DateTime<Utc> {
        self.time
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

    fn from_record(record: &csv::StringRecord) -> Result<TrackEntry, Box<dyn Error>> {
        let d = NaiveDate::parse_from_str(record.get(0).ok_or("missing date")?, "%Y%m%d")?;
        let t = NaiveTime::parse_from_str(record.get(1).ok_or("missing time")?, "%H%M")?;
        let time = Utc.from_utc_datetime(&NaiveDateTime::new(d, t));

        Ok(TrackEntry {
            time,
            indicator: match record.get(2).ok_or("missing indicator")?.trim() {
                "" => None,
                s => Some(Indicator::from_str(s)?),
            },
            status: record.get(3).ok_or("missing status")?.trim().parse()?,
            location: parse_location(
                record.get(4).ok_or("missing latitude")?.trim(),
                record.get(5).ok_or("missing longitude")?.trim(),
            )?,
            max_sustained_wind: record
                .get(6)
                .ok_or("max sustained wind")?
                .trim()
                .parse()
                .map_err(|_| "invalid max sustained wind")?,
            min_pressure: record
                .get(7)
                .ok_or("missing min_pressure")?
                .trim()
                .parse()
                .map_err(|_| "invalid min_pressure")?,
            wind_radii_34kts: parse_wind_radii(record, 8)
                .map_err(|_| "invalid wind_radii_34kts")?,
            wind_radii_50kts: parse_wind_radii(record, 12)
                .map_err(|_| "invalid wind_radii_50kts")?,
            wind_radii_64kts: parse_wind_radii(record, 16)
                .map_err(|_| "invalid wind_radii_64kts")?,
        })
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
        if s.len() != 1 {
            return Err(format!("invalid indicator: {}", s).into());
        }
        Indicator::from_char(s.chars().next().unwrap())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    fn from_record(record: &csv::StringRecord) -> Result<Header, Box<dyn Error>> {
        if record.len() != 4 {
            return Err(format!("storm header has {} columns, not 4.", record.len()).into());
        }

        let id = record
            .get(0)
            .ok_or("missing id")?
            .trim()
            .parse::<atcf::Id>()?;
        let name = match record.get(1).ok_or("missing name")?.trim() {
            "UNAMED" => None,
            v => Some(v.to_owned()),
        };
        let num_track_entries = record
            .get(2)
            .ok_or("missing num entries")?
            .trim()
            .parse::<usize>()?;
        Ok(Header {
            id,
            name,
            num_track_entries,
        })
    }
}
