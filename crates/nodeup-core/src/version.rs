use std::path::PathBuf;
use std::str::FromStr;
use std::fmt::{Display, Formatter};

#[derive(Eq, PartialEq)]
pub enum Version {
    Public(String)
}

#[derive(Clone, Eq, PartialEq)]
pub enum VersionSpec {
    Latest,
    Path(PathBuf),
    Specific(String)
}

impl Display for VersionSpec {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        match *self {
            VersionSpec::Latest => f.write_str("latest"),
            VersionSpec::Path(ref path) => path.to_string_lossy().fmt(f),
            VersionSpec::Specific(ref spec) => f.write_str(spec)
        }
    }
}

impl FromStr for VersionSpec {
    type Err = ::errors::ErrorKind;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(match src {
            "latest" => VersionSpec::Latest,
            // FIXME: recognize paths, validate version strings
            _ => VersionSpec::Specific(String::from(src))
        })
    }
}
