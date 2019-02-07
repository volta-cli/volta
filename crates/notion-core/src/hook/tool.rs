//! Types representing Notion Tool Hooks.

use std::ffi::OsString;
use std::io::Read;
use std::process::{Command, Stdio};

use crate::path::{ARCH, OS};
use cmdline_words_parser::StrExt;
use failure::Fail;
use notion_fail::{throw, FailExt, Fallible, ResultExt};
use semver::Version;

const ARCH_TEMPLATE: &'static str = "{{arch}}";
const OS_TEMPLATE: &'static str = "{{os}}";
const VERSION_TEMPLATE: &'static str = "{{version}}";

/// A hook for resolving the distro URL for a given tool version
#[derive(PartialEq, Debug)]
pub enum DistroHook {
    Prefix(String),
    Template(String),
    Bin(String),
}

impl DistroHook {
    /// Performs resolution of the distro URL based on the given
    /// version and file name
    pub fn resolve(&self, version: &Version, filename: &str) -> Fallible<String> {
        match self {
            &DistroHook::Prefix(ref prefix) => Ok(format!("{}{}", prefix, filename)),
            &DistroHook::Template(ref template) => Ok(template
                .replace(ARCH_TEMPLATE, ARCH)
                .replace(OS_TEMPLATE, OS)
                .replace(VERSION_TEMPLATE, &version.to_string())),
            &DistroHook::Bin(ref bin) => execute_binary(bin, Some(version.to_string())),
        }
    }
}

/// A hook for resolving the URL for metadata about a tool
#[derive(PartialEq, Debug)]
pub enum MetadataHook {
    Prefix(String),
    Template(String),
    Bin(String),
}

impl MetadataHook {
    /// Performs resolution of the metadata URL based on the given default file name
    pub fn resolve(&self, filename: &str) -> Fallible<String> {
        match self {
            &MetadataHook::Prefix(ref prefix) => Ok(format!("{}{}", prefix, filename)),
            &MetadataHook::Template(ref template) => Ok(template
                .replace(ARCH_TEMPLATE, ARCH)
                .replace(OS_TEMPLATE, OS)),
            &MetadataHook::Bin(ref bin) => execute_binary(bin, None),
        }
    }
}

/// Execute a shell command and return the trimmed stdout from that command
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

#[cfg(test)]
pub mod tests {
    use super::{DistroHook, MetadataHook};
    use crate::path::{ARCH, OS};
    use semver::Version;

    #[test]
    fn test_distro_prefix_resolve() {
        let prefix = "http://localhost/node/distro/";
        let filename = "node.tar.gz";
        let hook = DistroHook::Prefix(prefix.to_string());
        let version = Version::new(1, 0, 0);

        assert_eq!(
            hook.resolve(&version, filename)
                .expect("Could not resolve URL"),
            format!("{}{}", prefix, filename)
        );
    }

    #[test]
    fn test_distro_template_resolve() {
        let hook = DistroHook::Template(
            "http://localhost/node/{{os}}/{{arch}}/{{version}}/node.tar.gz".to_string(),
        );
        let version = Version::new(1, 0, 0);
        let expected = format!(
            "http://localhost/node/{}/{}/{}/node.tar.gz",
            OS,
            ARCH,
            version.to_string()
        );

        assert_eq!(
            hook.resolve(&version, "node.tar.gz")
                .expect("Could not resolve URL"),
            expected
        );
    }

    #[test]
    fn test_metadata_prefix_resolve() {
        let prefix = "http://localhost/node/index/";
        let filename = "index.json";
        let hook = MetadataHook::Prefix(prefix.to_string());

        assert_eq!(
            hook.resolve(filename).expect("Could not resolve URL"),
            format!("{}{}", prefix, filename)
        );
    }

    #[test]
    fn test_metadata_template_resolve() {
        let hook =
            MetadataHook::Template("http://localhost/node/{{os}}/{{arch}}/index.json".to_string());
        let expected = format!("http://localhost/node/{}/{}/index.json", OS, ARCH);

        assert_eq!(
            hook.resolve("index.json").expect("Could not resolve URL"),
            expected
        );
    }
}
