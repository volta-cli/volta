//! Types representing Volta Tool Hooks.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use crate::command::create_command;
use crate::error::{Context, ErrorKind, Fallible};
use crate::tool::{NODE_DISTRO_ARCH, NODE_DISTRO_OS};
use cmdline_words_parser::StrExt;
use dunce::canonicalize;
use lazy_static::lazy_static;
use log::debug;
use semver::Version;

const ARCH_TEMPLATE: &str = "{{arch}}";
const OS_TEMPLATE: &str = "{{os}}";
const VERSION_TEMPLATE: &str = "{{version}}";

lazy_static! {
    static ref REL_PATH: String = format!(".{}", std::path::MAIN_SEPARATOR);
    static ref REL_PATH_PARENT: String = format!("..{}", std::path::MAIN_SEPARATOR);
}

/// A hook for resolving the distro URL for a given tool version
#[derive(PartialEq, Debug)]
pub enum DistroHook {
    Prefix(String),
    Template(String),
    Bin { bin: String, base_path: PathBuf },
}

impl DistroHook {
    /// Performs resolution of the distro URL based on the given
    /// version and file name
    pub fn resolve(&self, version: &Version, filename: &str) -> Fallible<String> {
        match &self {
            DistroHook::Prefix(prefix) => Ok(format!("{}{}", prefix, filename)),
            DistroHook::Template(template) => Ok(template
                .replace(ARCH_TEMPLATE, NODE_DISTRO_ARCH)
                .replace(OS_TEMPLATE, NODE_DISTRO_OS)
                .replace(VERSION_TEMPLATE, &version.to_string())),
            DistroHook::Bin { bin, base_path } => {
                execute_binary(bin, base_path, Some(version.to_string()))
            }
        }
    }
}

/// A hook for resolving the URL for metadata about a tool
#[derive(PartialEq, Debug)]
pub enum MetadataHook {
    Prefix(String),
    Template(String),
    Bin { bin: String, base_path: PathBuf },
}

impl MetadataHook {
    /// Performs resolution of the metadata URL based on the given default file name
    pub fn resolve(&self, filename: &str) -> Fallible<String> {
        match &self {
            MetadataHook::Prefix(prefix) => Ok(format!("{}{}", prefix, filename)),
            MetadataHook::Template(template) => Ok(template
                .replace(ARCH_TEMPLATE, NODE_DISTRO_ARCH)
                .replace(OS_TEMPLATE, NODE_DISTRO_OS)),
            MetadataHook::Bin { bin, base_path } => execute_binary(bin, base_path, None),
        }
    }
}

/// Execute a shell command and return the trimmed stdout from that command
fn execute_binary(bin: &str, base_path: &Path, extra_arg: Option<String>) -> Fallible<String> {
    let mut trimmed = bin.trim().to_string();
    let mut words = trimmed.parse_cmdline_words();
    let cmd = match words.next() {
        Some(word) => {
            // Treat any path that starts with a './' or '../' as a relative path (using OS separator)
            if word.starts_with(REL_PATH.as_str()) || word.starts_with(REL_PATH_PARENT.as_str()) {
                canonicalize(base_path.join(word)).with_context(|| ErrorKind::HookPathError {
                    command: String::from(word),
                })?
            } else {
                PathBuf::from(word)
            }
        }
        None => {
            return Err(ErrorKind::InvalidHookCommand {
                command: String::from(bin.trim()),
            }
            .into())
        }
    };

    let mut args: Vec<OsString> = words.map(OsString::from).collect();
    if let Some(arg) = extra_arg {
        args.push(OsString::from(arg));
    }

    let mut command = create_command(cmd);
    command
        .args(&args)
        .current_dir(base_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    debug!("Running hook command: {:?}", command);
    let output = command
        .output()
        .with_context(|| ErrorKind::ExecuteHookError {
            command: String::from(bin.trim()),
        })?;

    if !output.status.success() {
        return Err(ErrorKind::HookCommandFailed {
            command: bin.trim().into(),
        }
        .into());
    }

    let url = String::from_utf8(output.stdout).with_context(|| ErrorKind::InvalidHookOutput {
        command: String::from(bin.trim()),
    })?;

    Ok(url.trim().to_string())
}

#[cfg(test)]
pub mod tests {
    use super::{DistroHook, MetadataHook};
    use crate::tool::{NODE_DISTRO_ARCH, NODE_DISTRO_OS};
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
            NODE_DISTRO_OS,
            NODE_DISTRO_ARCH,
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
        let expected = format!(
            "http://localhost/node/{}/{}/index.json",
            NODE_DISTRO_OS, NODE_DISTRO_ARCH
        );

        assert_eq!(
            hook.resolve("index.json").expect("Could not resolve URL"),
            expected
        );
    }
}
