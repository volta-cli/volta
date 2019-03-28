pub(crate) mod serial;

use std::fmt;
use std::str::FromStr;

use semver::{ReqParseError, Version, VersionReq};

use crate::error::ErrorDetails;
use notion_fail::{Fallible, ResultExt};

use self::serial::parse_requirements;

#[derive(Debug, Clone)]
pub enum VersionSpec {
    Latest,
    Semver(VersionReq),
    Exact(Version),
}

impl fmt::Display for VersionSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
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
        s.parse().with_context(version_parse_error(s))
    }

    pub fn parse_requirements(s: impl AsRef<str>) -> Fallible<VersionReq> {
        parse_requirements(s.as_ref()).with_context(version_parse_error(s))
    }

    pub fn parse_version(s: impl AsRef<str>) -> Fallible<Version> {
        Version::parse(s.as_ref()).with_context(version_parse_error(s))
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

fn version_parse_error<E, S>(version: S) -> impl FnOnce(&E) -> ErrorDetails
where
    E: std::error::Error,
    S: AsRef<str>,
{
    let version = version.as_ref().to_string();
    |_error: &E| ErrorDetails::VersionParseError { version }
}

// remove the leading 'v' from the version string, if present
fn trim_version(s: &str) -> &str {
    let s = s.trim();
    if s.starts_with('v') {
        s[1..].trim()
    } else {
        s
    }
}

// custom serialization and de-serialization for Version
// because Version doesn't work with serde out of the box
pub mod version_serde {
    use semver::Version;
    use serde::de::{Error, Visitor};
    use serde::{self, Deserializer, Serializer};
    use std::fmt;

    struct VersionVisitor;

    impl<'de> Visitor<'de> for VersionVisitor {
        type Value = Version;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("string")
        }

        // parse the version from the string
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Version::parse(super::trim_version(value)).map_err(Error::custom)
        }
    }

    pub fn serialize<S>(version: &Version, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&version.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(VersionVisitor)
    }

}

// custom serialization and de-serialization for Option<Version>
// because Version doesn't work with serde out of the box
pub mod option_version_serde {
    use semver::Version;
    use serde::de::Error;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(version: &Option<Version>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match version {
            Some(v) => s.serialize_str(&v.to_string()),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Version>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(v) = s {
            return Ok(Some(
                Version::parse(super::trim_version(&v)).map_err(Error::custom)?,
            ));
        }
        Ok(None)
    }

}
