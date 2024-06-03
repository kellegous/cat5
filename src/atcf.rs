use std::{error::Error, str::FromStr};

use serde::{de, ser};

#[derive(Debug, Copy, Clone)]
pub enum Basin {
    NorthAtlantic,
    CentralNorthPacific,
    EasternNorthPacific,
    WesternNorthPacific,
    NorthIndian,
    SouthernHemisphere,
}

impl Basin {
    pub fn to_str(&self) -> &str {
        match self {
            Basin::NorthAtlantic => "AL",
            Basin::CentralNorthPacific => "CP",
            Basin::EasternNorthPacific => "EP",
            Basin::WesternNorthPacific => "WP",
            Basin::NorthIndian => "IO",
            Basin::SouthernHemisphere => "SH",
        }
    }
}

impl FromStr for Basin {
    type Err = Box<dyn Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AL" => Ok(Basin::NorthAtlantic),
            "CP" => Ok(Basin::CentralNorthPacific),
            "EP" => Ok(Basin::EasternNorthPacific),
            "WP" => Ok(Basin::WesternNorthPacific),
            "IO" => Ok(Basin::NorthIndian),
            "SH" => Ok(Basin::SouthernHemisphere),
            _ => Err(format!("invalid basin: {}", s).into()),
        }
    }
}

impl std::fmt::Display for Basin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[derive(Debug)]
pub struct Id {
    basin: Basin,
    number: i32,
    year: i32,
}

impl FromStr for Id {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 8 {
            return Err("atcf id must be 8 characters".into());
        }
        Ok(Id {
            basin: s[..2].parse()?,
            number: s[2..4].parse()?,
            year: s[4..8].parse()?,
        })
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{:02}{:04}", self.basin, self.number, self.year)
    }
}

impl ser::Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let s = format!("{}", self);
        serializer.serialize_str(&s)
    }
}

impl<'de> de::Deserialize<'de> for Id {
    fn deserialize<D>(d: D) -> Result<Id, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        s.parse().map_err(de::Error::custom)
    }
}
