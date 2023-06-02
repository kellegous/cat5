use chrono::{DateTime, Utc};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::{self, DirEntry, WalkDir};

fn files_from<P: AsRef<Path>>(dir: P) -> Result<Vec<(PathBuf, DateTime<Utc>)>, Box<dyn Error>> {
	fn from_entry(
		r: walkdir::Result<DirEntry>,
	) -> Result<(PathBuf, DateTime<Utc>), Box<dyn Error>> {
		let entry = r?;
		let metadata = entry.metadata()?;
		let modified: DateTime<Utc> = metadata.modified()?.into();
		Ok((entry.path().to_owned(), modified))
	}

	WalkDir::new(&dir)
		.into_iter()
		.filter(|r| match r {
			Ok(e) => e.file_type().is_file(),
			_ => true,
		})
		.map(|r| from_entry(r))
		.collect()
}

fn main() -> Result<(), Box<dyn Error>> {
	let files = files_from("ui")?;
	let latest = files.iter().map(|(_, t)| t).max().unwrap();
	println!("{:?}, {}", files, latest);

	if !Command::new("make").status()?.success() {
		return Err("make failed".into());
	}
	Ok(())
}
