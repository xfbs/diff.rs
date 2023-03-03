use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use url::Url;

pub static CRATE_INFO_CACHE: Mutex<BTreeMap<String, Arc<CrateResponse>>> =
    Mutex::new(BTreeMap::new());
pub static CRATE_SOURCE_CACHE: Mutex<BTreeMap<String, Arc<CrateSource>>> =
    Mutex::new(BTreeMap::new());

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CrateResponse {
    //pub categories: BTreeSet<String>,
    #[serde(rename = "crate")]
    pub krate: CrateInfo,
    pub versions: Vec<VersionInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CrateInfo {
    pub categories: BTreeSet<String>,
    pub description: String,
    pub downloads: u64,
    pub exact_match: bool,
    pub homepage: Option<Url>,
    pub max_version: String,
    pub max_stable_version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
        drop(lock);

        Ok(info)
    }

    pub fn cached(name: &str) -> Option<Arc<CrateResponse>> {
        // check if we have it cached
        let lock = CRATE_INFO_CACHE.lock().unwrap();
        lock.get(name).cloned()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CrateSource {
    pub files: BTreeMap<String, String>,
}
