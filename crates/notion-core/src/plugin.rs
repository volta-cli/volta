use std::io::Read;
use std::process::{Command, Stdio};
use std::ffi::OsString;

use serial;

use failure;
use semver::{Version, VersionReq};
use serde_json;
use cmdline_words_parser::StrExt;

pub enum Resolve {
    Url(String),
    Bin(String)
}

impl Resolve {
    pub fn resolve(&self, req: &VersionReq) -> Result<Version, failure::Error> {
        match self {
            &Resolve::Url(_) => {
                unimplemented!()
            }

            &Resolve::Bin(ref bin) => {
                let mut bin = bin.trim().to_string();
                let mut words = bin.parse_cmdline_words();
                // FIXME: error for not having any commands
                let cmd = words.next().unwrap();
                let args: Vec<OsString> = words.map(|s| {
                    let mut os = OsString::new();
                    os.push(s);
                    os
                }).collect();
                let child = Command::new(cmd)
                    .args(&args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap(); // FIXME: error for failed spawn
                let response = ResolveResponse::from_reader(child.stdout.unwrap())?;
                eprintln!("response: {:?}", response);
                panic!("there's a bin plugin")
            }
        }
    }
}

#[derive(Debug)]
pub enum ResolveResponse {
    Url { url: String, version: Version },
    Stream { version: Version }
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
