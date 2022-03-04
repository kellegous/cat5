pub mod atcf;
pub mod geo;
pub mod hurdat2;
pub mod noaa;

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

pub struct DataDir {
	path: PathBuf,
}

impl DataDir {
	pub fn at<P: AsRef<Path>>(path: P) -> Result<DataDir, Box<dyn Error>> {
		let path = path.as_ref();
		if !path.exists() {
			fs::create_dir_all(path)?;
		}
		Ok(DataDir { path: path.into() })
	}

	pub fn download(&self, name: &str, url: &str) -> Result<fs::File, Box<dyn Error>> {
		let req = Client::new().get(url);
		let path = self.path.join(name);
		let res = if let Some(md) = Metadata::from_link(&path).ok() {
			req.header(IF_MODIFIED_SINCE, md.last_modified)
				.header(IF_NONE_MATCH, md.etag)
		} else {
			req
		}
		.send()?;

		match res.status() {
			StatusCode::OK => {
				// contruct new metadata from headers
				let md = Metadata::from_headers(res.headers())?;

				let content = res.bytes()?;
				let hash = hash_of(&content);

				// write hash file
				let dst = self.path.join(&hash);
				fs::File::create(&dst)?.write_all(&content)?;

				// write metadata
				md.to_path(&dst.with_extension("meta"))?;

				// TODO(knorton): I would prefer to just do a swap via symlink.
				if path.exists() {
					fs::remove_file(&path)?;
				}

				// symlink to the name file
				std::os::unix::fs::symlink(&hash, &path)?;
				Ok(fs::File::open(&path)?)
			}
			StatusCode::NOT_MODIFIED => Ok(fs::File::open(&path)?),
			s => Err(format!("status: {}", s).into()),
		}
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
	last_modified: String,
	etag: String,
}

impl Metadata {
	fn from_headers(headers: &HeaderMap) -> Result<Metadata, Box<dyn Error>> {
		let last_modified = match headers.get(LAST_MODIFIED) {
			Some(v) => v.to_str()?.to_owned(),
			None => String::new(),
		};
		let etag = match headers.get(ETAG) {
			Some(v) => v.to_str()?.to_owned(),
			None => String::new(),
		};
		Ok(Metadata {
			last_modified: last_modified,
			etag: etag,
		})
	}

	fn from_path<P: AsRef<Path>>(src: P) -> Result<Metadata, Box<dyn Error>> {
		Ok(serde_json::from_reader(fs::File::open(src)?)?)
	}

	fn from_link<P: AsRef<Path>>(link: P) -> Result<Metadata, Box<dyn Error>> {
		let dir = link.as_ref().parent().unwrap();
		Self::from_path(dir.join(fs::read_link(&link)?).with_extension("meta"))
	}

	fn to_path<P: AsRef<Path>>(&self, dst: P) -> Result<(), Box<dyn Error>> {
		Ok(serde_json::to_writer(fs::File::create(dst)?, self)?)
	}
}

fn hash_of(b: &[u8]) -> String {
	let mut h = Sha256::new();
	h.update(b);
	hex::encode(h.finalize().as_slice())
}
