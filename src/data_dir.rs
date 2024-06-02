use std::{
    error::Error,
    path::{Path, PathBuf},
};

use reqwest::{
    header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED},
    StatusCode,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::{
    fs,
    io::{self, AsyncReadExt, AsyncWriteExt},
};

#[derive(Debug)]
pub struct DataDir<'a> {
    client: &'a reqwest::Client,
    path: PathBuf,
}

impl<'a> DataDir<'a> {
    pub async fn create<P: AsRef<Path>>(
        client: &'a reqwest::Client,
        dir: P,
    ) -> Result<Self, Box<dyn Error>> {
        let dir = dir.as_ref();
        if !dir.exists() {
            fs::create_dir_all(dir).await?;
        }

        Ok(Self {
            client,
            path: dir.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn get_object(&self, name: &str) -> Object {
        Object {
            dir: self,
            name: name.to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct Object<'a> {
    dir: &'a DataDir<'a>,
    name: String,
}

impl<'a> Object<'a> {
    pub fn path(&self) -> PathBuf {
        self.dir.path.join(&self.name)
    }

    pub async fn fetch(self, url: &str, strategy: FetchStrategy) -> Result<Self, Box<dyn Error>> {
        let path = self.path();
        match strategy {
            FetchStrategy::Always => {
                self.download(url, &Metadata::empty()).await?;
                Ok(self)
            }
            FetchStrategy::IfMissing => {
                if !path.exists() {
                    self.download(url, &Metadata::empty()).await?;
                }
                Ok(self)
            }
            FetchStrategy::IfOutdated => {
                let md = Metadata::from_link(path).await?;
                self.download(url, &md).await?;
                Ok(self)
            }
        }
    }

    pub async fn open(&self) -> Result<fs::File, Box<dyn Error>> {
        let path = self.path();
        Ok(fs::File::open(&path).await?)
    }

    pub async fn create(&self) -> Result<fs::File, Box<dyn Error>> {
        let path = self.path();
        Ok(fs::File::create(&path).await?)
    }

    async fn download(&self, url: &str, md: &Metadata) -> Result<(), Box<dyn Error>> {
        let mut req = self.dir.client.get(url);
        if let Some(last_modified) = &md.last_modified {
            req = req.header(IF_MODIFIED_SINCE, last_modified);
        } else if let Some(etag) = &md.etag {
            req = req.header(IF_NONE_MATCH, etag);
        }

        let res = req.send().await?;
        match res.status() {
            StatusCode::OK => {
                let md = Metadata::from_headers(res.headers())?;

                let content = res.bytes().await?;

                let hash = hex::encode(Sha256::digest(content.as_ref()));

                let dest = self.dir.path.join(&hash);

                fs::File::create(&dest)
                    .await?
                    .write_all(content.as_ref())
                    .await?;

                md.to_writer(&mut fs::File::create(dest.with_extension("meta")).await?)
                    .await?;

                let link = self.dir.path.join(&self.name);
                symlink(&hash, &link).await?;
                Ok(())
            }
            StatusCode::NOT_MODIFIED => Ok(()),
            s => Err(format!("unexpected status code: {}", s).into()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FetchStrategy {
    IfMissing,
    IfOutdated,
    Always,
}

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    last_modified: Option<String>,
    etag: Option<String>,
}

impl Metadata {
    fn empty() -> Self {
        Self {
            last_modified: None,
            etag: None,
        }
    }

    fn from_headers(headers: &reqwest::header::HeaderMap) -> Result<Self, Box<dyn Error>> {
        let last_modified = match headers.get(LAST_MODIFIED) {
            Some(v) => Some(v.to_str()?.to_owned()),
            None => None,
        };

        let etag = match headers.get(ETAG) {
            Some(v) => Some(v.to_str()?.to_owned()),
            None => None,
        };

        Ok(Self {
            last_modified,
            etag,
        })
    }

    async fn from_link<P>(link: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let dir = link
            .as_ref()
            .parent()
            .ok_or("link has not parent directory")?;
        let path = fs::read_link(&link).await?;
        Self::from_reader(&mut fs::File::open(dir.join(path).with_extension("meta")).await?).await
    }

    async fn from_reader<R>(r: &mut R) -> Result<Self, Box<dyn Error>>
    where
        R: io::AsyncRead + Unpin,
    {
        let mut contents = vec![];
        r.read_to_end(&mut contents).await?;
        Ok(serde_json::from_slice(&contents)?)
    }

    async fn to_writer<W>(&self, w: &mut W) -> Result<(), Box<dyn Error>>
    where
        W: io::AsyncWrite + Unpin,
    {
        let contents = serde_json::to_vec(self)?;
        w.write_all(&contents).await?;
        Ok(())
    }
}

async fn symlink(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let tmp = dst.as_ref().with_extension("tmp");
    fs::symlink(src, &tmp).await?;
    fs::rename(&tmp, dst).await
}
