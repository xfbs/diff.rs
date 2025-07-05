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

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SummaryCategory {
    MostDownloaded,
    MostRecent,
    JustUpdated,
    RecentDownloads,
}

impl SummaryCategory {
    pub fn title(&self) -> &str {
        match self {
            SummaryCategory::MostDownloaded => "Most Downloaded",
            SummaryCategory::MostRecent => "New Crates",
            SummaryCategory::JustUpdated => "Just Updated",
            SummaryCategory::RecentDownloads => "Most Recent Downloads",
        }
    }
}

impl SummaryResponse {
    pub fn get(&self, cat: SummaryCategory) -> &Vec<CrateDetail> {
        match cat {
            SummaryCategory::JustUpdated => &self.just_updated,
            SummaryCategory::MostDownloaded => &self.most_downloaded,
            SummaryCategory::RecentDownloads => &self.most_recently_downloaded,
            SummaryCategory::MostRecent => &self.new_crates,
        }
    }
}

/// Create info struct, returned as part of the crates.io response.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrateDetail {
    pub id: String,
    pub max_version: Version,
    pub max_stable_version: Option<Version>,
    pub newest_version: Version,
    pub description: String,
    pub downloads: u64,
    pub recent_downloads: Option<u64>,
    pub exact_match: bool,
    pub homepage: Option<Url>,
    pub repository: Option<Url>,
    pub documentation: Option<Url>,
}

/// Crates.io response type for crate lookup
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrateResponse {
    //pub categories: BTreeSet<String>,
    #[serde(rename = "crate")]
    pub krate: CrateDetail,
    pub versions: Vec<VersionInfo>,
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
    #[serde(rename = "num")]
    pub version: Version,
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
            VersionId::Exact(version) => self.versions.iter().find(|v| v.version == version),
            VersionId::Named(VersionNamed::Latest) => self.versions.first(),
            VersionId::Named(VersionNamed::Previous) => {
                self.versions.get(1).or(self.versions.first())
            }
            VersionId::Requirement(req) => self
                .versions
                .iter()
                .filter(|v| req.matches(&v.version))
                .max_by_key(|v| &v.version),
        }
    }
}

impl VersionInfo {
    /// Get download URL for this crate
    ///
    /// We purposefully construct a URL here and don't use the one returned in the response,
    /// because we want to download it from the CDN instead of from the API (so it does not count
    /// towards crate downloads).
    pub fn download_url(&self) -> Result<Url> {
        let Self { krate, version, .. } = &self;
        let url = format!("https://static.crates.io/crates/{krate}/{krate}-{version}.crate");
        let url = url.parse()?;
        Ok(url)
    }

    /// Fetch a crate source for the given version.
    pub async fn fetch(&self) -> Result<CrateSource> {
        info!(
            "Fetching crate source for {} v{} from network",
            self.krate, self.version
        );
        let url = self.download_url()?;
        let response = Request::get(url.as_str()).send().await?;
        if !response.ok() {
            return Err(anyhow!("Error response: {}", response.status()));
        }

        let bytes: Bytes = response.binary().await?.into();
        let source = CrateSource::new(self.clone(), &bytes[..])?;

        Ok(source)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RepositoryInfo {
    pub repository: Url,
    pub vcs_info: CargoVcsInfo,
}

impl RepositoryInfo {
    pub fn url(&self) -> Option<Url> {
        if self.repository.as_str().starts_with("https://github.com/") {
            let mut url = Url::parse("https://codeload.github.com/").unwrap();
            url.path_segments_mut()
                .unwrap()
                .extend(self.repository.path_segments().unwrap())
                .extend(&["tar.gz", &self.vcs_info.git.sha1]);
            let url = format!("https://corsproxy.io/?{url}").parse().unwrap();
            return Some(url);
        }

        if self.repository.as_str().starts_with("https://gitlab.com/") {
            let mut url = self.repository.clone();
            url.path_segments_mut().unwrap().extend(&[
                "-",
                "archive",
                &format!("{}.tar.gz", self.vcs_info.git.sha1),
            ]);
            let url = format!("https://corsproxy.io/?{url}").parse().unwrap();
            return Some(url);
        }

        None
    }

    fn prefix(&self) -> String {
        let repo = self.repository.path().split('/').next_back().unwrap_or("");
        let mut prefix = format!("{repo}-{}/", self.vcs_info.git.sha1);

        if !self.vcs_info.path_in_vcs.is_empty() {
            prefix.push_str(&self.vcs_info.path_in_vcs);
            prefix.push('/');
        }

        prefix
    }

    pub async fn fetch(&self) -> Result<CrateSource> {
        let version = VersionInfo {
            checksum: vec![],
            dl_path: Default::default(),
            krate: "".into(),
            yanked: false,
            version: "0.0.0".parse().unwrap(),
        };
        let url = self
            .url()
            .ok_or(anyhow::anyhow!("cannot get repository URL"))?;
        let response = Request::get(url.as_str()).send().await?;
        if !response.ok() {
            return Err(anyhow!("Error response: {}", response.status()));
        }

        let bytes = response.binary().await?;
        let prefix = self.prefix();
        Ok(CrateSource {
            version,
            files: CrateSource::parse_archive(&prefix, &bytes[..], false)?,
        })
    }
}

type FileContents = BTreeMap<Utf8PathBuf, Bytes>;

/// Crate source
///
/// This is parsed from the gzipped tarball that crates.io serves for every crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrateSource {
    pub version: VersionInfo,
    pub files: FileContents,
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
        // compute hash
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();

        // make sure hash matches
        if hash[..] != version.checksum[..] {
            return Err(CrateSourceError::HashsumMismatch {
                expected: version.checksum.clone(),
                got: hash[..].to_vec(),
            });
        }

        let prefix = format!("{}-{}/", version.krate, version.version);
        let source = CrateSource {
            version,
            files: Self::parse_archive(&prefix, data, true)?,
        };

        Ok(source)
    }

    /// Parse gzipped archive.
    fn parse_archive(
        prefix: &str,
        data: &[u8],
        error_outside_prefix: bool,
    ) -> Result<FileContents, CrateSourceError> {
        let mut data = GzDecoder::new(data);
        let mut archive = Archive::new(&mut data);
        let mut files = FileContents::default();

        // this is the path prefix we expect in the archive.
        for entry in archive.entries()? {
            let mut entry = entry?;

            //debug!("{} {:?}", entry.path().unwrap().display(), entry.header().entry_type());
            if !entry.header().entry_type().is_file() {
                continue;
            }

            // make path encoding error explicit
            let bytes = entry.path_bytes();
            let path = std::str::from_utf8(&bytes)?;
            let path = match path.strip_prefix(prefix) {
                Some(path) => path,
                None if error_outside_prefix => {
                    return Err(CrateSourceError::InvalidPrefix {
                        path: path.to_string(),
                        prefix: prefix.into(),
                    })
                }
                None => continue,
            };

            let path: Utf8PathBuf = path.into();

            // read data
            let mut data = vec![];
            entry.read_to_end(&mut data)?;

            debug!("Storing path {path} ({} bytes)", data.len());
            // store data
            files.insert(path, data.into());
        }

        Ok(files)
    }

    /// Get [`CargoVcsInfo`] from the crate sources.
    pub fn cargo_vcs_info(&self) -> Result<CargoVcsInfo, CargoVcsInfoError> {
        let raw = self
            .files
            .get(Utf8Path::new(".cargo_vcs_info.json"))
            .ok_or(CargoVcsInfoError::Missing)?;
        let decoded = serde_json::from_slice(raw)?;
        Ok(decoded)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CargoVcsInfoError {
    #[error("missing .cargo_vcs_info.json")]
    Missing,
    #[error("cannot decode .cargo_vcs_info.json")]
    Decode(#[from] serde_json::Error),
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct CargoVcsInfo {
    git: CargoGitInfo,
    path_in_vcs: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct CargoGitInfo {
    sha1: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct FileDiff {
    /// Diff in this file
    pub changes: Vec<(ChangeTag, Bytes)>,
    /// Ranges of lines to show for each file
    pub context_ranges: Vec<ChunkInfo>,
    // Redundant - alternativly take from files
    pub summary: Changes,
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
            left.version.krate, left.version.version, right.version.krate, right.version.version
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
                        // cnt for determining start idx of hunk, wanna start before this line, so do not count current line
                        offsets.push((index, insertions, deletions));
                        deletions += 1;
                    }
                    ChangeTag::Insert => {
                        offsets.push((index, insertions, deletions));
                        insertions += 1;
                    }
                }
            }

            // compute ranges to show
            let mut ranges = vec![];
            let mut last_hunk = (0..0, 0, 0);

            for (offset, ins, del) in offsets.iter() {
                let hunk_start = offset.saturating_sub(CONTEXT_LINES);
                let left_start = hunk_start.saturating_sub(*ins);
                let right_start = hunk_start.saturating_sub(*del);

                let hunk = (
                    hunk_start..*offset + CONTEXT_LINES + 1,
                    left_start,
                    right_start,
                );
                let overlaps_with_last_hunk = hunk.0.start.max(last_hunk.0.start)
                    <= hunk.0.end.min(last_hunk.0.end) + CONTEXT_LINES;
                if overlaps_with_last_hunk {
                    last_hunk = (last_hunk.0.start..hunk.0.end, last_hunk.1, last_hunk.2);
                } else {
                    if last_hunk.0.end != 0 {
                        ranges.push(last_hunk.clone().into());
                    }
                    last_hunk = hunk;
                }
            }

            // Push the last hunk we've computed if any
            if last_hunk.0.end != 0 {
                ranges.push(last_hunk.into())
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
                    summary: Changes {
                        added: insertions as u64,
                        removed: deletions as u64,
                    },
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

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ChunkInfo {
    pub range: Range<usize>,
    pub left_start: usize,
    pub right_start: usize,
}

impl From<(Range<usize>, usize, usize)> for ChunkInfo {
    fn from((range, left_start, right_start): (Range<usize>, usize, usize)) -> Self {
        ChunkInfo {
            range,
            left_start,
            right_start,
        }
    }
}

impl ChunkInfo {
    pub fn start(&self) -> usize {
        self.range.start
    }
    pub fn end(&self) -> usize {
        self.range.end
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
        debug!("Inserting {path} with changes {changes:?}");
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
