//! Types representing Notion plugins.

use std::ffi::OsString;
use std::io::Read;
use std::process::{Command, Stdio};

use distro::Distro;

use cmdline_words_parser::StrExt;
use notion_fail::{FailExt, Fallible, ResultExt};
use semver::{Version, VersionReq};
use serde_json;

pub mod serial;

/// A Node version resolution plugin.
pub enum ResolvePlugin {
    /// Resolves a Tool version by sending it to a URL and receiving the
    /// resolution in the response.
    Url(String),

    /// Resolves a Tool version by passing it to an executable and
    /// receiving the resolution in the process's stdout stream.
    Bin(String),
}

#[derive(Fail, Debug)]
#[fail(display = "Invalid plugin command: '{}'", command)]
pub struct InvalidCommandError {
    command: String,
}

impl ResolvePlugin {
    /// Performs resolution of a Tool version based on the given semantic
    /// versioning requirements.
    pub fn resolve<D: Distro>(&self, _matching: &VersionReq) -> Fallible<D> {
        match self {
            &ResolvePlugin::Url(_) => unimplemented!(),

            &ResolvePlugin::Bin(ref bin) => {
                let mut trimmed = bin.trim().to_string();
                let mut words = trimmed.parse_cmdline_words();
                let cmd = if let Some(word) = words.next() {
                    word
                } else {
                    throw!(
                        InvalidCommandError {
                            command: String::from(bin.trim()),
                        }.unknown()
                    );
                };
                let args: Vec<OsString> = words
                    .map(|s| {
                        let mut os = OsString::new();
                        os.push(s);
                        os
                    })
                    .collect();
                let child = Command::new(cmd)
                    .args(&args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unknown()?;
                let response = ResolveResponse::from_reader(child.stdout.unwrap())?;
                match response {
                    ResolveResponse::Url { version, url } => D::remote(version, &url),
                    ResolveResponse::Stream { version: _version } => {
                        unimplemented!("bin plugin produced a stream")
                    }
                }
            }
        }
    }
}

/// A response from the Node version resolution plugin.
#[derive(Debug)]
pub enum ResolveResponse {
    /// A plugin response indicating that the Node installer for the resolved version
    /// can be downloaded from the specified URL.
    Url { version: Version, url: String },

    /// A plugin response indicating that the Node installer for the resolved version
    /// is being delivered via the stderr stream of the plugin process.
    Stream { version: Version },
}

impl ResolveResponse {
    /// Reads and parses a response from a Node version resolution plugin.
    pub fn from_reader<R: Read>(reader: R) -> Fallible<Self> {
        let serial: serial::ResolveResponse = serde_json::from_reader(reader).unknown()?;
        Ok(serial.into_resolve_response()?)
    }
}

/// A plugin listing the available versions of Node.
pub enum LsRemote {
    Url(String),
    Bin(String),
}

/// A plugin for publishing Notion events.
pub enum Publish {
    /// Reports an event by sending a POST request to a URL.
    Url(String),

    /// Reports an event by forking a process and sending the event by IPC.
    Bin(String),
}
