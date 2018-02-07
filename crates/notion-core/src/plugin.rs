use std::io::Read;

use serial;

use failure;
use semver::Version;
use serde_json;

pub enum Resolve {
    Url(String),
    Bin(String)
}

#[derive(Debug)]
pub enum ResolveResponse {
    Url { url: String, version: Version }
}

impl ResolveResponse {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, failure::Error> {
        let serial: serial::plugin::ResolveResponse = serde_json::from_reader(reader)?;
        Ok(serial.into_resolve_response()?)
    }
}

pub enum LsRemote {
    Url(String),
    Bin(String)
}
