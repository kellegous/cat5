use chrono::{DateTime, Utc};
use std::error::Error;
use std::path::Path;
use std::process::Command;
use walkdir::{DirEntry, WalkDir};

fn get_build_sha() -> Result<String, Box<dyn Error>> {
	let output = Command::new("git").output()?;
	Ok(String::from_utf8(output.stdout)?)
}

fn needs_update<P: AsRef<Path>>(dst: P, src: P) -> Result<bool, Box<dyn Error>> {
	fn to_modified_time(r: walkdir::Result<DirEntry>) -> Result<DateTime<Utc>, Box<dyn Error>> {
		let modified: DateTime<Utc> = r?.metadata()?.modified()?.into();
		Ok(modified)
	}
	let src_times = WalkDir::new(&src)
		.into_iter()
		.filter(|r| match r {
			Ok(e) => e.file_type().is_file(),
			_ => true,
		})
		.map(|r| to_modified_time(r))
		.collect::<Result<Vec<_>, _>>()?;
	let src_latest = *src_times.iter().max().unwrap(); // fixme
	let dst_latest: DateTime<Utc> = dst.as_ref().metadata()?.modified()?.into();
	Ok(src_latest > dst_latest)
}

fn build_ui() -> Result<(), Box<dyn Error>> {
	if needs_update("dist/index.js", "ui")? {
		if !Command::new("npm")
			.args(["run", "build-prd"])
			.status()?
			.success()
		{
			return Err("webpack build failed".into());
		}
	}
	Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
	let build_sha = get_build_sha()?;
	build_ui()?;
	println!("cargo:rustc-env=BUILD_SHA={}", build_sha);
	Ok(())
}
