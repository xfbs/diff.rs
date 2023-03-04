use anyhow::{anyhow, Result};
use flate2::bufread::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::sync::{Arc, Mutex};
use tar::Archive;
use url::Url;

pub static CRATE_INFO_CACHE: Mutex<BTreeMap<String, Arc<CrateResponse>>> =
    Mutex::new(BTreeMap::new());
pub static CRATE_SOURCE_CACHE: Mutex<BTreeMap<(String, String), Arc<CrateSource>>> =
    Mutex::new(BTreeMap::new());

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrateResponse {
    //pub categories: BTreeSet<String>,
    #[serde(rename = "crate")]
    pub krate: CrateInfo,
    pub versions: Vec<VersionInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrateInfo {
    pub categories: BTreeSet<String>,
    pub description: String,
    pub downloads: u64,
    pub exact_match: bool,
    pub homepage: Option<Url>,
    pub max_version: String,
    pub max_stable_version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct VersionInfo {
    pub checksum: String,
    #[serde(rename = "crate")]
    pub krate: String,
    pub crate_size: Option<u64>,
    pub dl_path: String,
    pub downloads: u64,
    pub license: Option<String>,
    pub yanked: bool,
    pub id: u64,
    pub num: String,
}

impl CrateInfo {
    pub async fn fetch(name: &str) -> Result<CrateResponse> {
        let base: Url = "https://crates.io/api/v1/crates/".parse()?;
        let url = base.join(name)?;
        let response = reqwest::get(url.as_str()).await?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Error response: {}", response.status()))
        }
    }

    pub async fn fetch_cached(name: &str) -> Result<Arc<CrateResponse>> {
        if let Some(info) = Self::cached(name) {
            return Ok(info);
        }

        // fetch it
        let info = CrateInfo::fetch(name).await?;
        let info = Arc::new(info);

        // save back into cache
        let mut lock = CRATE_INFO_CACHE.lock().unwrap();
        lock.insert(name.to_string(), info.clone());
        Ok(info)
    }

    pub fn cached(name: &str) -> Option<Arc<CrateResponse>> {
        // check if we have it cached
        let lock = CRATE_INFO_CACHE.lock().unwrap();
        lock.get(name).cloned()
    }
}

impl VersionInfo {
    pub fn cached(&self) -> Option<Arc<CrateSource>> {
        let lock = CRATE_SOURCE_CACHE.lock().unwrap();
        lock.get(&(self.krate.clone(), self.num.clone())).cloned()
    }

    pub async fn fetch_cached(&self) -> Result<Arc<CrateSource>> {
        if let Some(source) = self.cached() {
            return Ok(source);
        }

        let source = self.fetch().await?;
        let source = Arc::new(source);

        let mut lock = CRATE_SOURCE_CACHE.lock().unwrap();
        lock.insert((self.krate.clone(), self.num.clone()), source.clone());
        Ok(source)
    }

    pub async fn fetch(&self) -> Result<CrateSource> {
        let base: Url = "https://crates.io/".parse()?;
        let url = base.join(&self.dl_path)?;
        let response = reqwest::get(url.as_str()).await?;
        if response.status().is_success() {
            let bytes = response.bytes().await?;
            Ok(CrateSource::parse_compressed(&bytes[..])?)
        } else {
            Err(anyhow!("Error response: {}", response.status()))
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct CrateSource {
    pub files: BTreeMap<String, String>,
}

impl CrateSource {
    pub fn parse_compressed(data: &[u8]) -> Result<Self> {
        let mut decoder = GzDecoder::new(data);
        Self::parse_archive(&mut decoder)
    }

    pub fn parse_archive(data: &mut dyn Read) -> Result<Self> {
        let mut archive = Archive::new(data);
        let mut source = Self::default();
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = String::from_utf8_lossy(&entry.path_bytes()).to_string();
            let path: String = path.chars().skip_while(|c| *c != '/').skip(1).collect();
            let data = std::io::read_to_string(&mut entry)?;
            source.add(&path, data);
        }
        Ok(source)
    }

    pub fn add(&mut self, path: &str, data: String) {
        self.files.insert(path.to_string(), data);
    }
}
