use std::{
    convert::Infallible,
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

use crate::version::VersionId;

#[derive(Clone, PartialEq)]
pub enum MaybeParsed<T> {
    Parsed(T),
    String(String),
}

impl<T: FromStr> FromStr for MaybeParsed<T> {
    type Err = Infallible;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(T::from_str(input)
            .map(Self::Parsed)
            .unwrap_or_else(|_| MaybeParsed::String(input.to_string())))
    }
}

impl<T: Display> Display for MaybeParsed<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Parsed(i) => i.fmt(f),
            Self::String(i) => i.fmt(f),
        }
    }
}

impl<T: ToString> From<MaybeParsed<T>> for String {
    fn from(value: MaybeParsed<T>) -> Self {
        match value {
            MaybeParsed::Parsed(i) => i.to_string(),
            MaybeParsed::String(i) => i,
        }
    }
}

impl<T> From<String> for MaybeParsed<T> {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<semver::Version> for MaybeParsed<VersionId> {
    fn from(value: semver::Version) -> Self {
        Self::Parsed(value.into())
    }
}

impl From<VersionId> for MaybeParsed<VersionId> {
    fn from(value: VersionId) -> Self {
        Self::Parsed(value)
    }
}
