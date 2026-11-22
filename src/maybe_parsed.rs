use crate::version::VersionId;
use std::{
    convert::Infallible,
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

/// Route parameter which may or may not parse as `T`.
///
/// Always keeps the original string, so that a segment which happens to parse (for example a
/// directory named `1`, which parses as the version requirement `^1`) can be reproduced verbatim.
#[derive(Clone, PartialEq, Debug)]
pub struct MaybeParsed<T> {
    raw: String,
    parsed: Option<T>,
}

impl<T> MaybeParsed<T> {
    /// Original string of this segment.
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Parsed value, if the segment parsed.
    pub fn parsed(&self) -> Option<&T> {
        self.parsed.as_ref()
    }
}

impl<T: FromStr> FromStr for MaybeParsed<T> {
    type Err = Infallible;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            raw: input.to_string(),
            parsed: input.parse().ok(),
        })
    }
}

impl<T> Display for MaybeParsed<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.raw.fmt(f)
    }
}

impl From<VersionId> for MaybeParsed<VersionId> {
    fn from(value: VersionId) -> Self {
        Self {
            raw: value.to_string(),
            parsed: Some(value),
        }
    }
}

impl From<semver::Version> for MaybeParsed<VersionId> {
    fn from(value: semver::Version) -> Self {
        VersionId::from(value).into()
    }
}
