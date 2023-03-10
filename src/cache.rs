use crate::data::*;
use anyhow::Result;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// Crate response cache
pub struct CrateResponseCache(Mutex<BTreeMap<String, Arc<CrateResponse>>>);

/// Global crate response cache instance
pub static CRATE_RESPONSE_CACHE: CrateResponseCache = CrateResponseCache::new();

impl CrateResponseCache {
    /// Create new, empty cache
    pub const fn new() -> Self {
        CrateResponseCache(Mutex::new(BTreeMap::new()))
    }

    /// Lookup in cache or fetch
    pub async fn fetch_cached(&self, name: &str) -> Result<Arc<CrateResponse>> {
        if let Some(info) = self.cached(name) {
            return Ok(info);
        }

        // fetch it
        let info = CrateResponse::fetch(name).await?;
        let info = Arc::new(info);

        // save back into cache
        self.cache(info.clone());

        Ok(info)
    }

    /// Store in cache
    fn cache<T: Into<Arc<CrateResponse>>>(&self, response: T) {
        let mut lock = self.0.lock().unwrap();
        let response: Arc<CrateResponse> = response.into();
        lock.insert(response.krate.id.clone(), response);
    }

    /// Lookup in cache
    pub fn cached(&self, name: &str) -> Option<Arc<CrateResponse>> {
        // check if we have it cached
        let lock = self.0.lock().unwrap();
        lock.get(name).cloned()
    }
}

#[test]
fn test_crate_response_cache_missing() {
    let cache = CrateResponseCache::new();
    assert!(cache.cached("serde").is_none());
}

#[test]
fn test_crate_response_cache_store() {
    let cache = CrateResponseCache::new();
    assert!(cache.cached("serde").is_none());
    let crate_response = Arc::new(CrateResponse {
        krate: CrateInfo {
            id: "serde".into(),
            max_version: "0.1.0".into(),
            max_stable_version: Some("0.1.0".into()),
        },
        versions: Default::default(),
    });
    cache.cache(crate_response.clone());
    assert_eq!(crate_response, cache.cached("serde").unwrap());
}

/// Crate source cache
pub struct CrateSourceCache(Mutex<BTreeMap<(String, String), Arc<CrateSource>>>);

/// Global crate source cache instance
pub static CRATE_SOURCE_CACHE: CrateSourceCache = CrateSourceCache::new();

impl CrateSourceCache {
    /// Create new, empty cache
    pub const fn new() -> Self {
        CrateSourceCache(Mutex::new(BTreeMap::new()))
    }

    /// Lookup in cache or fetch
    pub async fn fetch_cached(&self, version: &VersionInfo) -> Result<Arc<CrateSource>> {
        if let Some(source) = self.cached(&version) {
            return Ok(source);
        }

        // fetch it
        let source = version.fetch().await?;
        let source = Arc::new(source);

        // save back into cache
        self.cache(source.clone());

        Ok(source)
    }

    /// Store in cache
    fn cache<T: Into<Arc<CrateSource>>>(&self, source: T) {
        let mut lock = self.0.lock().unwrap();
        let source: Arc<CrateSource> = source.into();
        lock.insert(
            (source.version.krate.clone(), source.version.num.clone()),
            source,
        );
    }

    /// Lookup in cache
    pub fn cached(&self, version: &VersionInfo) -> Option<Arc<CrateSource>> {
        // check if we have it cached
        let lock = self.0.lock().unwrap();
        lock.get(&(version.krate.clone(), version.num.clone()))
            .cloned()
    }
}

#[test]
fn test_crate_source_cache_missing() {
    let cache = CrateSourceCache::new();
    let version = VersionInfo {
        checksum: "abc".into(),
        dl_path: "/path".into(),
        krate: "serde".into(),
        num: "0.1.0".into(),
        yanked: false,
    };
    assert!(cache.cached(&version).is_none());
}

#[test]
fn test_crate_source_cache_store() {
    let cache = CrateSourceCache::new();
    let version = VersionInfo {
        checksum: "abc".into(),
        dl_path: "/path".into(),
        krate: "serde".into(),
        num: "0.1.0".into(),
        yanked: false,
    };
    assert!(cache.cached(&version).is_none());
    let source = Arc::new(CrateSource {
        version: version.clone(),
        files: Default::default(),
    });
    cache.cache(source.clone());
    assert_eq!(source, cache.cached(&version).unwrap());
}
