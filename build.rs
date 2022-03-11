use chrono::{DateTime, NaiveDateTime, Utc};
use handlebars::Handlebars;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::{DirEntry, WalkDir};

fn get_build_sha() -> Result<String, Box<dyn Error>> {
	let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
	Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn needs_update<P: AsRef<Path>>(dst: P, src: P) -> Result<bool, Box<dyn Error>> {
	fn to_modified_time(r: walkdir::Result<DirEntry>) -> Result<DateTime<Utc>, Box<dyn Error>> {
		let modified: DateTime<Utc> = r?.metadata()?.modified()?.into();
		Ok(modified)
	}

	let src = src.as_ref();
	let dst = dst.as_ref();

	if !dst.exists() {
		return Ok(true);
	}

	let src_latest = if src.is_dir() {
		let src_times = WalkDir::new(&src)
			.into_iter()
			.filter(|r| match r {
				Ok(e) => e.file_type().is_file(),
				_ => true,
			})
			.map(|r| to_modified_time(r))
			.collect::<Result<Vec<_>, _>>()?;
		let zero = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc);
		*src_times.iter().max().unwrap_or(&zero)
	} else {
		src.metadata()?.modified()?.into()
	};
	let dst_latest: DateTime<Utc> = dst.metadata()?.modified()?.into();
	Ok(src_latest > dst_latest)
}

fn build_js() -> Result<(), Box<dyn Error>> {
	// TODO(knorton): This is broken on incremental builds because the profile isn't
	// tagged in the build artifact in anyway.
	let profile = std::env::var("PROFILE").unwrap_or("release".to_owned());
	let target = if profile == "debug" {
		"build-dev"
	} else {
		"build-prd"
	};

	if Command::new("npm")
		.args(["run", target])
		.status()?
		.success()
	{
		Ok(())
	} else {
		Err("webpack build failed".into())
	}
}

fn build_html<P: AsRef<Path>>(
	dst: P,
	src: P,
	vars: &HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
	let mut hbs = Handlebars::new();
	hbs.set_strict_mode(true);
	hbs.register_template_file("index", src)?;
	hbs.render_to_write("index", vars, fs::File::create(&dst)?)?;
	Ok(())
}

fn maybe_build<P: AsRef<Path>, F>(dst: P, src: P, f: F) -> Result<(), Box<dyn Error>>
where
	F: FnOnce() -> Result<(), Box<dyn Error>>,
{
	if needs_update(&dst, &src)? {
		f()
	} else {
		Ok(())
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	let build_sha = get_build_sha()?;

	let mut varz = HashMap::new();
	varz.insert("sha".to_owned(), build_sha.clone());

	maybe_build("dist/index.js", "ui", build_js)?;
	maybe_build("dist/index.html", "ui/index.hbs", || {
		build_html("dist/index.html", "ui/index.hbs", &varz)
	})?;

	println!("cargo:rustc-env=BUILD_SHA={}", build_sha);
	Ok(())
}
