//! Types representing Notion plugins.

use std::io::Read;
use std::process::{Command, Stdio};
use std::ffi::OsString;

use serial;
use installer::node::Installer;

use failure;
use semver::{Version, VersionReq};
use serde_json;
use cmdline_words_parser::StrExt;

pub enum Resolve {
    Url(String),
    Bin(String)
}

#[derive(Fail, Debug)]
#[fail(display = "Invalid plugin command: '{}'", command)]
pub struct InvalidCommandError {
    command: String
}

impl Resolve {
    pub fn resolve(&self, _req: &VersionReq) -> Result<Installer, failure::Error> {
        match self {
            &Resolve::Url(_) => {
                unimplemented!()
            }

            &Resolve::Bin(ref bin) => {
                let mut trimmed = bin.trim().to_string();
                let mut words = trimmed.parse_cmdline_words();
                let cmd = if let Some(word) = words.next() {
                    word
                } else {
                    return Err(InvalidCommandError {
                        command: String::from(bin.trim())
                    }.into());
                };
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
                    .spawn()?;
                let response = ResolveResponse::from_reader(child.stdout.unwrap())?;
                match response {
                    ResolveResponse::Url { version, url } => {
                        Installer::remote(version, &url)
                    }
                    ResolveResponse::Stream { version: _version } => {
                        unimplemented!("bin plugin produced a stream")
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ResolveResponse {
    Url { version: Version, url: String },
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
