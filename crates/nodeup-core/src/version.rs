use std::path::PathBuf;
use std::str::FromStr;

// const LATEST_URL: &'static str = "http://nodejs.org/dist/latest/SHASUMS256.txt";

pub enum Version {
    Public(String)
}

pub enum VersionSpec {
    Latest,
    Path(PathBuf),
    Specific(String)
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
