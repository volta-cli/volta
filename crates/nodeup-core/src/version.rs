use std::path::PathBuf;

const LATEST_URL: &'static str = "http://nodejs.org/dist/latest/SHASUMS256.txt";

pub enum Version {
    Public(String)
}

pub enum VersionSpec {
    Latest,
    Path(PathBuf),
    Specific(String)
}
