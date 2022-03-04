use std::error::Error;
use std::str::FromStr;

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

impl Id {
	pub fn year(&self) -> i32 {
		self.year
	}

	pub fn number(&self) -> i32 {
		self.number
	}

	pub fn basin(&self) -> Basin {
		self.basin
	}
}

impl FromStr for Id {
	type Err = Box<dyn Error>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.len() == 8 {
			Ok(Id {
				basin: (&s[..2]).parse()?,
				number: (&s[2..4]).parse()?,
				year: (&s[4..8]).parse()?,
			})
		} else {
			Err("atcf id must be 8 characters".into())
		}
	}
}

impl std::fmt::Display for Id {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}{:02}{:04}", self.basin, self.number, self.year)
	}
}
