use crate::version::{VersionId, VersionNamed};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use flate2::bufread::GzDecoder;
use gloo_net::http::Request;
use log::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use similar::{ChangeTag, TextDiff};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    ops::Range,
    rc::Rc,
    sync::Arc,
};
use subslice_offset::SubsliceOffset;
use tar::Archive;
use url::Url;

/// Crates.io response type for crate search
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SearchResponse {
    pub crates: Vec<CrateDetail>,
}

/// Crates.io response for summary fetch
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SummaryResponse {
    pub just_updated: Vec<CrateDetail>,
    pub most_downloaded: Vec<CrateDetail>,
    pub most_recently_downloaded: Vec<CrateDetail>,
    pub new_crates: Vec<CrateDetail>,
}

impl SummaryResponse {
    pub fn get(&self, cat: &super::components::SummaryCategory) -> &Vec<CrateDetail> {
        match cat {
            super::components::SummaryCategory::JustUpdated => &self.just_updated,
            super::components::SummaryCategory::MostDownloaded => &self.most_downloaded,
            super::components::SummaryCategory::RecentDownloads => &self.most_recently_downloaded,
            super::components::SummaryCategory::MostRecent => &self.new_crates,
        }
    }
}

/// Create info struct, returned as part of the crates.io response.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrateDetail {
    pub id: String,
    pub max_version: String,
    pub max_stable_version: Option<String>,
    pub description: String,
    pub downloads: u64,
    pub exact_match: bool,
    pub homepage: Option<Url>,
}

/// Crates.io response type for crate lookup
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrateResponse {
    //pub categories: BTreeSet<String>,
    #[serde(rename = "crate")]
    pub krate: CrateInfo,
    pub versions: Vec<VersionInfo>,
}

/// Create info struct, returned as part of the crates.io response.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrateInfo {
    pub id: String,
    pub max_version: String,
    pub max_stable_version: Option<String>,
    //pub categories: BTreeSet<String>,
    //pub description: String,
    //pub downloads: u64,
    //pub exact_match: bool,
    //pub homepage: Option<Url>,
}

/// Version info struct, returned as part of the crates.io response.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct VersionInfo {
    #[serde(with = "hex")]
    pub checksum: Vec<u8>,
    #[serde(rename = "crate")]
    pub krate: String,
    pub dl_path: String,
    pub yanked: bool,
    pub num: Version,
    //pub id: u64,
    //pub crate_size: Option<u64>,
    //pub downloads: u64,
    //pub license: Option<String>,
}

impl CrateResponse {
    /// Fetch a CrateResponse for the given crate.
    pub async fn fetch(name: &str) -> Result<Self> {
        info!("Fetching crate metadata for {name} from network");
        let base: Url = "https://crates.io/api/v1/crates/".parse()?;
        let url = base.join(name)?;
        let response = Request::get(url.as_str()).send().await?;
        if response.ok() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Error response: {}", response.status()))
        }
    }

    pub fn version(&self, version: VersionId) -> Option<&VersionInfo> {
        match version {
            VersionId::Exact(version) => self.versions.iter().find(|v| v.num == version),
            VersionId::Named(VersionNamed::Latest) => self.versions.first(),
            VersionId::Named(VersionNamed::Previous) => self.versions.get(1),
            VersionId::Requirement(req) => self
                .versions
                .iter()
                .filter(|v| req.matches(&v.num))
                .max_by_key(|v| &v.num),
        }
    }
}

impl VersionInfo {
    /// Fetch a crate source for the given version.
    pub async fn fetch(&self) -> Result<CrateSource> {
        info!(
            "Fetching crate source for {} v{} from network",
            self.krate, self.num
        );
        let url = format!(
            "https://static.crates.io/crates/{}/{}-{}.crate",
            self.krate, self.krate, self.num
        );
        let url: Url = url.parse()?;
        let response = Request::get(url.as_str()).send().await?;
        if !response.ok() {
            return Err(anyhow!("Error response: {}", response.status()));
        }

        let bytes: Bytes = response.binary().await?.into();
        let source = CrateSource::new(self.clone(), &bytes[..])?;

        Ok(source)
    }
}

/// Crate source
///
/// This is parsed from the gzipped tarball that crates.io serves for every crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrateSource {
    pub version: VersionInfo,
    pub files: BTreeMap<Utf8PathBuf, Bytes>,
}

#[derive(thiserror::Error, Debug)]
pub enum CrateSourceError {
    /// We get an expected hashsum in the crate info response from crates.io. When
    /// we download a crate, we verify that the data we got matches this. If not,
    /// return an error here.
    #[error("hashsum mismatch in crate response: expected {expected:02x?} but got {got:02x?}")]
    HashsumMismatch { expected: Vec<u8>, got: Vec<u8> },

    /// These errors can be caused by the decompression (flate2 crate) or the untarring (tar
    /// crate).
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// In tar archives, paths are represented as raw bytes. We expect that these are valid UTF-8
    /// encoded strings. If this is not the case, we return an error. This is safer than using
    /// something like [`String::from_utf8_lossy`], because an attacker could place two files with
    /// invalid characters which would result in the same (lossy) path, thereby hiding the presence
    /// of a file from the user interface.
    #[error("error decoding path as utf8")]
    PathEncoding(#[from] std::str::Utf8Error),

    /// Crate tar archives contain files predixed under the path `<crate>-<version>`. There should
    /// not be any other files in this archive. If we encounter a file with a different path
    /// prefix, we return an error here. Those files would otherwise be invisible to the user
    /// interface.
    #[error("encountered invalid prefix in path {path} (expected {prefix})")]
    InvalidPrefix { path: String, prefix: String },
}

impl CrateSource {
    /// Create empty crate source for the given version.
    pub fn new(version: VersionInfo, data: &[u8]) -> Result<Self, CrateSourceError> {
        let mut source = CrateSource {
            version,
            files: Default::default(),
        };

        source.parse_compressed(data)?;
        Ok(source)
    }

    /// Parse gzipped tarball returned by crates.io.
    fn parse_compressed(&mut self, data: &[u8]) -> Result<(), CrateSourceError> {
        // compute hash
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();

        // make sure hash matches
        if hash[..] != self.version.checksum[..] {
            return Err(CrateSourceError::HashsumMismatch {
                expected: self.version.checksum.clone(),
                got: hash[..].to_vec(),
            });
        }

        let mut decoder = GzDecoder::new(data);
        self.parse_archive(&mut decoder)?;
        Ok(())
    }

    /// Parse archive.
    fn parse_archive(&mut self, data: &mut dyn Read) -> Result<(), CrateSourceError> {
        let mut archive = Archive::new(data);

        // this is the path prefix we expect in the archive.
        let prefix = format!("{}-{}/", self.version.krate, self.version.num);

        for entry in archive.entries()? {
            let mut entry = entry?;

            // make path encoding error explicit
            let bytes = entry.path_bytes();
            let path = std::str::from_utf8(&bytes)?;
            let path = match path.strip_prefix(&prefix) {
                Some(path) => path,
                None => {
                    return Err(CrateSourceError::InvalidPrefix {
                        path: path.to_string(),
                        prefix,
                    })
                }
            };
            let path: Utf8PathBuf = path.into();

            // read data
            let mut data = vec![];
            entry.read_to_end(&mut data)?;

            // store data
            self.add(&path, data);
        }
        Ok(())
    }

    /// Add a single file to crate source.
    fn add<T: Into<Bytes>>(&mut self, path: &Utf8Path, data: T) {
        self.files.insert(path.into(), data.into());
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct FileDiff {
    /// Diff in this file
    pub changes: Vec<(ChangeTag, Bytes)>,
    /// Ranges of lines to show for each file
    pub context_ranges: Vec<Range<usize>>,
}

/// Precomputed diff data
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionDiff {
    /// Left crate source that is diffed
    pub left: Arc<CrateSource>,
    /// Right crate source that is diffed
    pub right: Arc<CrateSource>,
    /// Files in this version diff
    pub files: BTreeMap<Utf8PathBuf, FileDiff>,
    /// Summaries of files and folders
    pub summary: BTreeMap<Utf8PathBuf, (usize, usize)>,

    pub tree: Entry,
}

/// How many lines of context to show in a diff
const CONTEXT_LINES: usize = 3;

impl VersionDiff {
    /// Generate diff data
    pub fn new(left: Arc<CrateSource>, right: Arc<CrateSource>) -> Self {
        info!(
            "Computing diff for {} version {} and {} version {}",
            left.version.krate, left.version.num, right.version.krate, right.version.num
        );

        let mut entry = Entry::default();
        let mut files = BTreeMap::new();
        let mut summary: BTreeMap<Utf8PathBuf, (usize, usize)> = BTreeMap::new();

        // intersection of file paths in both left and right crate sources
        let file_paths: BTreeSet<&Utf8Path> = left
            .files
            .keys()
            .chain(right.files.keys())
            .map(|s| s.as_path())
            .collect();

        // compute diffs
        for path in file_paths.into_iter() {
            info!("Computing diff for {path}");

            // lookup files, default to empty bytes
            let left = left.files.get(path).cloned().unwrap_or_default();
            let right = right.files.get(path).cloned().unwrap_or_default();

            // generate text diff
            let diff = TextDiff::from_lines(&left[..], &right[..]);

            // collect changes
            let changes: Vec<_> = diff
                .iter_all_changes()
                .map(|change| {
                    // soo... we do an awkward little dance here. out data is a Bytes struct, which we
                    // can cheaply get subslices from. the diff algorithm gets a &[u8] and every
                    // change gives us a &[u8]. now, we want to figure out what the offset of this
                    // &[u8] was from the original bytes, so that we can call .slice() on it to get a
                    // cheap reference-counted bytes rather than having to clone it. so we use the
                    // subslice_offset crate which lets us do exactly that.
                    let value = change.value();
                    let value = [&left, &right]
                        .iter()
                        .find_map(|b| {
                            b[..]
                                .subslice_offset(value)
                                .map(|index| b.slice(index..index + value.len()))
                        })
                        .unwrap();
                    (change.tag(), value)
                })
                .collect();

            let mut offsets = vec![];
            let mut insertions = 0;
            let mut deletions = 0;

            for (index, (tag, _)) in changes.iter().enumerate() {
                match tag {
                    ChangeTag::Equal => {}
                    ChangeTag::Delete => {
                        deletions += 1;
                        offsets.push(index);
                    }
                    ChangeTag::Insert => {
                        insertions += 1;
                        offsets.push(index);
                    }
                }
            }

            // compute ranges to show
            let mut ranges = vec![];
            let mut last_hunk = 0..0;

            for offset in offsets.iter() {
                let hunk = offset.saturating_sub(CONTEXT_LINES)..*offset + CONTEXT_LINES + 1;
                let overlaps_with_last_hunk =
                    hunk.start.max(last_hunk.start) <= hunk.end.min(last_hunk.end);
                if overlaps_with_last_hunk {
                    last_hunk = last_hunk.start..hunk.end;
                } else {
                    if last_hunk.end != 0 {
                        ranges.push(last_hunk.clone());
                    }
                    last_hunk = hunk;
                }
            }

            // Push the last hunk we've computed if any
            if last_hunk.end != 0 {
                ranges.push(last_hunk)
            }

            // compute additions
            for path in path.ancestors() {
                let summary = summary.entry(path.into()).or_default();
                summary.0 += insertions;
                summary.1 += deletions;
            }

            entry.insert(
                path,
                Changes {
                    added: insertions as u64,
                    removed: deletions as u64,
                },
            );

            files.insert(
                path.into(),
                FileDiff {
                    changes,
                    context_ranges: ranges,
                },
            );
        }

        VersionDiff {
            left,
            right,
            files,
            summary,
            tree: entry,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Changes {
    pub added: u64,
    pub removed: u64,
}

impl std::ops::Add for Changes {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self.added += rhs.added;
        self.removed += rhs.removed;
        self
    }
}

impl std::ops::AddAssign for Changes {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    #[default]
    Unchanged,
    Added,
    Deleted,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Item {
    File,
    Dir(BTreeMap<String, Rc<Entry>>),
}

impl Item {
    pub fn is_dir(&self) -> bool {
        matches!(self, Item::Dir(_))
    }
}

impl Default for Item {
    fn default() -> Self {
        Self::Dir(Default::default())
    }
}

#[derive(Clone, PartialEq, Default, Debug, Eq)]
pub struct Entry {
    pub name: String,
    pub item: Item,
    pub changes: Changes,
    pub state: State,
}

impl Entry {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn insert(&mut self, path: &Utf8Path, changes: Changes) {
        let mut entry = self;

        for component in path.components() {
            entry.changes += changes;

            let component = match component {
                Utf8Component::RootDir => continue,
                Utf8Component::Normal(path) => path,
                Utf8Component::CurDir => unreachable!(),
                Utf8Component::ParentDir => unreachable!(),
                Utf8Component::Prefix(_) => unreachable!(),
            };

            entry = Rc::make_mut(
                match &mut entry.item {
                    Item::File => unreachable!(),
                    Item::Dir(entries) => entries,
                }
                .entry(component.to_string())
                .or_insert_with(|| Rc::new(Entry::new(component.to_string()))),
            );
        }

        entry.changes = changes;
        entry.item = Item::File;
    }
}
