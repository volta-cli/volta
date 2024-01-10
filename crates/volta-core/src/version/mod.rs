use std::fmt;
use std::str::FromStr;

use crate::error::{Context, ErrorKind, Fallible, VoltaError};
use node_semver::{Range, Version};

mod serial;

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum VersionSpec {
    /// No version specified (default)
    #[default]
    None,

    /// SemVer Range
    Semver(Range),

    /// Exact Version
    Exact(Version),

    /// Arbitrary Version Tag
    Tag(VersionTag),
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum VersionTag {
    /// The 'latest' tag, a special case that exists for all packages
    Latest,

    /// The 'lts' tag, a special case for Node
    Lts,

    /// An arbitrary tag version
    Custom(String),
}

impl fmt::Display for VersionSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionSpec::None => write!(f, "<default>"),
            VersionSpec::Semver(req) => req.fmt(f),
            VersionSpec::Exact(version) => version.fmt(f),
            VersionSpec::Tag(tag) => tag.fmt(f),
        }
    }
}

impl fmt::Display for VersionTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionTag::Latest => write!(f, "latest"),
            VersionTag::Lts => write!(f, "lts"),
            VersionTag::Custom(s) => s.fmt(f),
        }
    }
}

impl FromStr for VersionSpec {
    type Err = VoltaError;

    fn from_str(s: &str) -> Fallible<Self> {
        if let Ok(version) = parse_version(s) {
            Ok(VersionSpec::Exact(version))
        } else if let Ok(req) = parse_requirements(s) {
            Ok(VersionSpec::Semver(req))
        } else {
            s.parse().map(VersionSpec::Tag)
        }
    }
}

impl FromStr for VersionTag {
    type Err = VoltaError;

    fn from_str(s: &str) -> Fallible<Self> {
        if s == "latest" {
            Ok(VersionTag::Latest)
        } else if s == "lts" {
            Ok(VersionTag::Lts)
        } else {
            Ok(VersionTag::Custom(s.into()))
        }
    }
}

pub fn parse_requirements(s: impl AsRef<str>) -> Fallible<Range> {
    let s = s.as_ref();
    serial::parse_requirements(s)
        .with_context(|| ErrorKind::VersionParseError { version: s.into() })
}

pub fn parse_version(s: impl AsRef<str>) -> Fallible<Version> {
    let s = s.as_ref();
    s.parse()
        .with_context(|| ErrorKind::VersionParseError { version: s.into() })
}

// remove the leading 'v' from the version string, if present
fn trim_version(s: &str) -> &str {
    let s = s.trim();
    match s.strip_prefix('v') {
        Some(stripped) => stripped,
        None => s,
    }
}

// custom serialization and de-serialization for Version
// because Version doesn't work with serde out of the box
pub mod version_serde {
    use node_semver::Version;
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
    use node_semver::Version;
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

// custom deserialization for HashMap<String, Version>
// because Version doesn't work with serde out of the box
pub mod hashmap_version_serde {
    use super::version_serde;
    use node_semver::Version;
    use serde::{self, Deserialize, Deserializer};
    use std::collections::HashMap;

    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "version_serde::deserialize")] Version);

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, Version>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let m = HashMap::<String, Wrapper>::deserialize(deserializer)?;
        Ok(m.into_iter().map(|(k, Wrapper(v))| (k, v)).collect())
    }
}
