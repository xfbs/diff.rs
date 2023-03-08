use anyhow::{anyhow, Result};
use flate2::bufread::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tar::Archive;
use url::Url;

/// Crates.io response type for crate lookup
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrateResponse {
    //pub categories: BTreeSet<String>,
    #[serde(rename = "crate")]
    pub krate: CrateInfo,
    pub versions: Vec<VersionInfo>,
}

#[cfg(test)]
#[tokio::test]
async fn test_crate_response_decode() {
    let serde: CrateResponse =
        serde_json::from_slice(include_bytes!("../data/serde.json")).unwrap();
    assert_eq!(serde.krate.id, "serde");
    assert!(!serde.versions.is_empty());

    let axum: CrateResponse = serde_json::from_slice(include_bytes!("../data/axum.json")).unwrap();
    assert_eq!(axum.krate.id, "axum");
    assert!(!axum.versions.is_empty());

    let reqwest: CrateResponse =
        serde_json::from_slice(include_bytes!("../data/reqwest.json")).unwrap();
    assert_eq!(reqwest.krate.id, "reqwest");
    assert!(!reqwest.versions.is_empty());
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrateInfo {
    pub id: String,
    pub max_version: String,
    pub max_stable_version: String,
    //pub categories: BTreeSet<String>,
    //pub description: String,
    //pub downloads: u64,
    //pub exact_match: bool,
    //pub homepage: Option<Url>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct VersionInfo {
    pub checksum: String,
    #[serde(rename = "crate")]
    pub krate: String,
    pub dl_path: String,
    pub yanked: bool,
    pub num: String,
    //pub id: u64,
    //pub crate_size: Option<u64>,
    //pub downloads: u64,
    //pub license: Option<String>,
}

impl CrateResponse {
    pub async fn fetch(name: &str) -> Result<Self> {
        let base: Url = "https://crates.io/api/v1/crates/".parse()?;
        let url = base.join(name)?;
        let response = reqwest::get(url.as_str()).await?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Error response: {}", response.status()))
        }
    }
}

impl VersionInfo {
    pub async fn fetch(&self) -> Result<CrateSource> {
        let base: Url = "https://crates.io/".parse()?;
        let url = base.join(&self.dl_path)?;
        let response = reqwest::get(url.as_str()).await?;
        if response.status().is_success() {
            let bytes = response.bytes().await?;
            let mut source = CrateSource::new(self.clone());
            source.parse_compressed(&bytes[..])?;
            Ok(source)
        } else {
            Err(anyhow!("Error response: {}", response.status()))
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrateSource {
    pub version: VersionInfo,
    pub files: BTreeMap<String, String>,
}

impl CrateSource {
    pub fn new(version: VersionInfo) -> Self {
        CrateSource {
            version,
            files: Default::default(),
        }
    }

    pub fn parse_compressed(&mut self, data: &[u8]) -> Result<()> {
        let mut decoder = GzDecoder::new(data);
        self.parse_archive(&mut decoder)?;
        Ok(())
    }

    pub fn parse_archive(&mut self, data: &mut dyn Read) -> Result<()> {
        let mut archive = Archive::new(data);
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = String::from_utf8_lossy(&entry.path_bytes()).to_string();
            let path: String = path.chars().skip_while(|c| *c != '/').skip(1).collect();
            let mut data = vec![];
            entry.read_to_end(&mut data)?;
            let data = String::from_utf8_lossy(&data).into_owned();
            self.add(&path, data);
        }
        Ok(())
    }

    pub fn add(&mut self, path: &str, data: String) {
        self.files.insert(path.to_string(), data);
    }
}
