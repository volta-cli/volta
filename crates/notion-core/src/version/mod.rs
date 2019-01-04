pub(crate) mod serial;

use std::fmt;
use std::str::FromStr;

use semver::{ReqParseError, SemVerError, Version, VersionReq};

use notion_fail::{ExitCode, Fallible, NotionFail, ResultExt};

use self::serial::parse_requirements;

#[derive(Debug, Clone)]
pub enum VersionSpec {
    Latest,
    Semver(VersionReq),
    Exact(Version),
}

impl fmt::Display for VersionSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            VersionSpec::Latest => write!(f, "latest"),
            VersionSpec::Semver(ref req) => req.fmt(f),
            VersionSpec::Exact(ref version) => version.fmt(f),
        }
    }
}

impl Default for VersionSpec {
    fn default() -> Self {
        VersionSpec::Latest
    }
}

impl VersionSpec {
    pub fn exact(version: &Version) -> Self {
        VersionSpec::Exact(version.clone())
    }

    pub fn parse(s: impl AsRef<str>) -> Fallible<Self> {
        let s = s.as_ref();
        s.parse()
            .with_context(VersionParseError::from_req_parse_error)
    }

    pub fn parse_requirements(s: impl AsRef<str>) -> Fallible<VersionReq> {
        parse_requirements(s.as_ref()).with_context(VersionParseError::from_req_parse_error)
    }

    pub fn parse_version(s: impl AsRef<str>) -> Fallible<Version> {
        Version::parse(s.as_ref()).with_context(VersionParseError::from_semver_error)
    }
}

impl FromStr for VersionSpec {
    type Err = ReqParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "latest" {
            return Ok(VersionSpec::Latest);
        }

        if let Ok(ref exact) = VersionSpec::parse_version(s) {
            Ok(VersionSpec::exact(exact))
        } else {
            Ok(VersionSpec::Semver(parse_requirements(s)?))
        }
    }
}

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "{}", error)]
#[notion_fail(code = "NoVersionMatch")]
pub(crate) struct VersionParseError {
    pub(crate) error: String,
}

impl VersionParseError {
    fn from_req_parse_error(error: &ReqParseError) -> Self {
        VersionParseError {
            error: error.to_string(),
        }
    }

    fn from_semver_error(error: &SemVerError) -> Self {
        VersionParseError {
            error: error.to_string(),
        }
    }
}
