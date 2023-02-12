use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use url::Url;

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

pub async fn crate_info(name: &str) -> Result<CrateResponse> {
    let base: Url = "https://crates.io/api/v1/crates/".parse()?;
    let url = base.join(name)?;
    let response = reqwest::get(url.as_str()).await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(anyhow!("Error response: {}", response.status()))
    }
}
