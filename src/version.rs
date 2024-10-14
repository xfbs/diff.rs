use semver::{Error, Version, VersionReq};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};
use strum::EnumString;

#[derive(Debug, PartialEq, Eq, EnumString, Clone, strum::Display)]
#[cfg_attr(test, derive(test_strategy::Arbitrary))]
#[strum(serialize_all = "kebab-case")]
pub enum VersionNamed {
    Latest,
    Previous,
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(test_strategy::Arbitrary))]
pub enum VersionId {
    Named(VersionNamed),
    #[cfg_attr(test, weight(0))]
    Exact(Version),
    #[cfg_attr(test, weight(0))]
    Requirement(VersionReq),
}

impl FromStr for VersionId {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Ok(named) = VersionNamed::from_str(input) {
            return Ok(named.into());
        }

        if let Ok(exact) = Version::from_str(input) {
            return Ok(exact.into());
        }

        Ok(VersionReq::from_str(input)?.into())
    }
}

impl Display for VersionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Named(named) => Display::fmt(named, f),
            Self::Exact(version) => Display::fmt(version, f),
            Self::Requirement(req) => Display::fmt(req, f),
        }
    }
}

macro_rules! from {
    ($ty:ty, $fn:expr) => {
        impl From<$ty> for VersionId {
            fn from(version: $ty) -> Self {
                $fn(version)
            }
        }
    };
}

from!(Version, Self::Exact);
from!(VersionReq, Self::Requirement);
from!(VersionNamed, Self::Named);

#[test]
fn can_parse_version_id() {
    assert_eq!(
        "latest".parse::<VersionId>().unwrap(),
        VersionId::Named(VersionNamed::Latest)
    );
    assert_eq!(
        "previous".parse::<VersionId>().unwrap(),
        VersionId::Named(VersionNamed::Previous)
    );
    assert_eq!(
        "0.1.0".parse::<VersionId>().unwrap(),
        VersionId::Exact("0.1.0".parse().unwrap()),
    );
}
