use crate::DfResult;
use bytes::Bytes;
use std::io::Read;
use url::Url;

#[derive(Debug, Clone)]
pub struct Puller {
    /// Maximum cache size in bytes
    pub max_cache_size: usize,
    /// Whether to allow access to the OS filesystem through 'file://'
    pub allow_local_fs: bool,
    // cache: HashMap<Url, Bytes>,
}

impl Default for Puller {
    fn default() -> Self {
        Self {
            max_cache_size: Self::DEFAULT_MAX_CACHE_SIZE, // 1gb cache size
            allow_local_fs: true,
        }
    }
}

impl Puller {
    /// Default cache size limit: 1GB
    const DEFAULT_MAX_CACHE_SIZE: usize = 1024 * 1024 * 1024;

    /// Make an http request
    async fn make_request(&self, url: Url) -> DfResult<reqwest::Response> {
        log::info!("pulling '{url}', scheme '{}'", url.scheme());

        // make http request
        Ok(reqwest::get(url.clone()).await?)
    }

    /// Read a local file and return its contents as a [`Bytes`]
    fn read_local_file(&self, path: &str) -> DfResult<Bytes> {
        log::info!("reading local file '{}'", path);
        let mut f = std::fs::File::open(path)?;
        let mut buf: Vec<u8> = vec![];
        f.read_to_end(&mut buf)?;
        Ok(buf.into())
    }

    /// Read a local file to a [`String`]
    fn read_local_file_str(&self, path: &str) -> DfResult<String> {
        log::info!("reading local file '{}' to string", path);
        let mut f = std::fs::File::open(path)?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        Ok(buf)
    }

    /// Pull bytes from a URL as a [`Bytes`]
    pub async fn pull_bytes(&mut self, url: Url) -> DfResult<Bytes> {
        if url.scheme() == "file" && self.allow_local_fs {
            self.read_local_file(url.path().trim_start_matches('/')) // trim starting slashes
        } else {
            Ok(self.make_request(url).await?.bytes().await?)
        }
    }

    /// Pull a [`String`] from a URL
    pub async fn pull_str(&mut self, url: Url) -> DfResult<String> {
        if url.scheme() == "file" && self.allow_local_fs {
            self.read_local_file_str(url.path().trim_start_matches('/')) // trim starting slashes
        } else {
            Ok(self.make_request(url).await?.text().await?)
        }
    }
}
