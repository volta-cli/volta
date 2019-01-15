//! Types representing Notion plugins.

use std::ffi::OsString;
use std::io::Read;
use std::process::{Command, Stdio};

use path::{ARCH, OS};

use cmdline_words_parser::StrExt;
use notion_fail::{FailExt, Fallible, ResultExt};
use semver::Version;

pub(crate) mod serial;

const ARCH_TEMPLATE: &'static str = "{arch}";
const OS_TEMPLATE: &'static str = "{os}";
const VERSION_TEMPLATE: &'static str = "{version}";

/// A Hook for resolving the distro URL for a given Tool Version
#[derive(PartialEq, Debug)]
pub enum ToolDistroHook {
    Prefix(String),
    Template(String),
    Bin(String),
}

impl ToolDistroHook {
    /// Performs resolution of the Distro URL based on the given
    /// Version and File Name
    pub fn resolve(&self, version: &Version, filename: String) -> Fallible<String> {
        match self {
            &ToolDistroHook::Prefix(ref prefix) => Ok(format!("{}{}", prefix, filename)),
            &ToolDistroHook::Template(ref template) => Ok(template
                .replace(ARCH_TEMPLATE, ARCH)
                .replace(OS_TEMPLATE, OS)
                .replace(VERSION_TEMPLATE, &version.to_string())),
            &ToolDistroHook::Bin(ref bin) => execute_binary(bin, Some(version.to_string())),
        }
    }
}

/// A Hook for resolving the URL for metadata about a Tool
#[derive(PartialEq, Debug)]
pub enum ToolMetadataHook {
    Prefix(String),
    Template(String),
    Bin(String),
}

impl ToolMetadataHook {
    /// Performs resolution of the Metadata URL based on the given default File Name
    pub fn resolve(&self, filename: String) -> Fallible<String> {
        match self {
            &ToolMetadataHook::Prefix(ref prefix) => Ok(format!("{}{}", prefix, filename)),
            &ToolMetadataHook::Template(ref template) => Ok(template
                .replace(ARCH_TEMPLATE, ARCH)
                .replace(OS_TEMPLATE, OS)),
            &ToolMetadataHook::Bin(ref bin) => execute_binary(bin, None),
        }
    }
}

fn execute_binary(bin: &str, extra_arg: Option<String>) -> Fallible<String> {
    let mut trimmed = bin.trim().to_string();
    let mut words = trimmed.parse_cmdline_words();
    let cmd = if let Some(word) = words.next() {
        word
    } else {
        throw!(InvalidCommandError {
            command: String::from(bin.trim()),
        }
        .unknown())
    };
    let mut args: Vec<OsString> = words.map(OsString::from).collect();

    if let Some(arg) = extra_arg {
        args.push(OsString::from(arg));
    }

    let child = Command::new(cmd)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unknown()?;

    let mut url = String::new();
    child.stdout.unwrap().read_to_string(&mut url).unknown()?;
    Ok(url.trim().to_string())
}

#[derive(Fail, Debug)]
#[fail(display = "Invalid hook command: '{}'", command)]
pub struct InvalidCommandError {
    command: String,
}

/// A plugin for publishing Notion events.
#[derive(PartialEq, Debug)]
pub enum Publish {
    /// Reports an event by sending a POST request to a URL.
    Url(String),

    /// Reports an event by forking a process and sending the event by IPC.
    Bin(String),
}
