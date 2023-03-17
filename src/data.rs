use anyhow::{anyhow, Result};
use bytes::Bytes;
use flate2::bufread::GzDecoder;
use log::*;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, Read};
use std::ops::Range;
use std::sync::Arc;
use subslice_offset::SubsliceOffset;
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
    /// Fetch a CrateResponse for the given crate.
    pub async fn fetch(name: &str) -> Result<Self> {
        info!("Fetching crate metadata for {name} from network");
        let base: Url = "https://crates.io/api/v1/crates/".parse()?;
        let url = base.join(name)?;
        let response = reqwest::get(url.as_str()).await?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("Error response: {}", response.status()))
        }
    }

    pub fn version(&self, version: &str) -> Option<&VersionInfo> {
        self.versions.iter().find(|v| v.num == version)
    }
}

impl VersionInfo {
    /// Fetch a crate source for the given version.
    pub async fn fetch(&self) -> Result<CrateSource> {
        info!(
            "Fetching crate source for {} v{} from network",
            self.krate, self.num
        );
        let base: Url = "https://crates.io/".parse()?;
        let url = base.join(&self.dl_path)?;
        let response = reqwest::get(url.as_str()).await?;
        if response.status().is_success() {
            let bytes = response.bytes().await?;
            let mut source = CrateSource::new(self.clone());
            source.parse_compressed(&mut &bytes[..])?;
            Ok(source)
        } else {
            Err(anyhow!("Error response: {}", response.status()))
        }
    }
}

/// Crate source
///
/// This is parsed from the gzipped tarball that crates.io serves for every crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrateSource {
    pub version: VersionInfo,
    pub files: BTreeMap<String, Bytes>,
}

impl CrateSource {
    /// Create empty crate source for the given version.
    pub fn new(version: VersionInfo) -> Self {
        CrateSource {
            version,
            files: Default::default(),
        }
    }

    /// Parse gzipped tarball returned by crates.io.
    pub fn parse_compressed(&mut self, data: &mut dyn BufRead) -> Result<()> {
        let mut decoder = GzDecoder::new(data);
        self.parse_archive(&mut decoder)?;
        Ok(())
    }

    /// Parse archive.
    pub fn parse_archive(&mut self, data: &mut dyn Read) -> Result<()> {
        let mut archive = Archive::new(data);
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = String::from_utf8_lossy(&entry.path_bytes()).to_string();
            let path: String = path.chars().skip_while(|c| *c != '/').skip(1).collect();

            // read data
            let mut data = vec![];
            entry.read_to_end(&mut data)?;

            // store data
            self.add(&path, data);
        }
        Ok(())
    }

    /// Add a single file to crate source.
    pub fn add<T: Into<Bytes>>(&mut self, path: &str, data: T) {
        self.files.insert(path.to_string(), data.into());
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
    pub files: BTreeMap<String, FileDiff>,
    /// Summaries of files and folders
    pub summary: BTreeMap<String, (usize, usize)>,
}

/// How many lines of context to show in a diff
const CONTEXT_LINES: usize = 3;

impl VersionDiff {
    /// Generate diff data
    pub fn new<T1: Into<Arc<CrateSource>>, T2: Into<Arc<CrateSource>>>(
        left: T1,
        right: T2,
    ) -> Self {
        let left = left.into();
        let right = right.into();
        info!(
            "Computing diff for {} version {} and {}",
            left.version.krate, left.version.num, right.version.num
        );

        // intersection of file paths in both left and right crate sources
        let file_paths: BTreeSet<&str> = left
            .files
            .keys()
            .chain(right.files.keys())
            .map(|s| s.as_str())
            .collect();

        let mut files = BTreeMap::new();
        let mut summary: BTreeMap<String, (usize, usize)> = BTreeMap::new();

        // compute diffs
        for path in file_paths.into_iter() {
            compute_file_diff(&left, path, &right, &mut summary, &mut files);
        }

        VersionDiff {
            left,
            right,
            files,
            summary,
        }
    }
}

fn compute_file_diff(
    left: &Arc<CrateSource>,
    path: &str,
    right: &Arc<CrateSource>,
    summary: &mut BTreeMap<String, (usize, usize)>,
    files: &mut BTreeMap<String, FileDiff>,
) {
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

    // Indexes of lines that contain an actual diff
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

    let ranges = compute_hunk_ranges(offsets);

    // compute additions
    for segment in path.split('/') {
        let end = path.subslice_offset(segment).unwrap() + segment.len();
        let path = path[0..end].to_string();
        let mut summary = summary.entry(path).or_default();
        summary.0 += insertions;
        summary.1 += deletions;
    }

    files.insert(
        path.to_string(),
        FileDiff {
            changes,
            context_ranges: ranges,
        },
    );
}

/// Compute contiguous ranges of changes given their `offsets`
fn compute_hunk_ranges(offsets: Vec<usize>) -> Vec<Range<usize>> {
    // TODO merge ranges that are <= CONTEXT_LINES away from each other
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
    ranges
}
