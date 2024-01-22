//! Types representing Volta Tool Hooks.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use crate::command::create_command;
use crate::error::{Context, ErrorKind, Fallible};
use crate::hook::RegistryFormat;
use crate::tool::{NODE_DISTRO_ARCH, NODE_DISTRO_OS};
use cmdline_words_parser::parse_posix;
use dunce::canonicalize;
use log::debug;
use node_semver::Version;
use once_cell::sync::Lazy;

const ARCH_TEMPLATE: &str = "{{arch}}";
const OS_TEMPLATE: &str = "{{os}}";
const VERSION_TEMPLATE: &str = "{{version}}";
const EXTENSION_TEMPLATE: &str = "{{ext}}";
const FILENAME_TEMPLATE: &str = "{{filename}}";

static REL_PATH: Lazy<String> = Lazy::new(|| format!(".{}", std::path::MAIN_SEPARATOR));
static REL_PATH_PARENT: Lazy<String> = Lazy::new(|| format!("..{}", std::path::MAIN_SEPARATOR));

/// A hook for resolving the distro URL for a given tool version
#[derive(PartialEq, Eq, Debug)]
pub enum DistroHook {
    Prefix(String),
    Template(String),
    Bin { bin: String, base_path: PathBuf },
}

impl DistroHook {
    /// Performs resolution of the distro URL based on the given version and file name
    pub fn resolve(&self, version: &Version, filename: &str) -> Fallible<String> {
        let extension = calculate_extension(filename).unwrap_or("");

        match &self {
            DistroHook::Prefix(prefix) => Ok(format!("{}{}", prefix, filename)),
            DistroHook::Template(template) => Ok(template
                .replace(ARCH_TEMPLATE, NODE_DISTRO_ARCH)
                .replace(OS_TEMPLATE, NODE_DISTRO_OS)
                .replace(EXTENSION_TEMPLATE, extension)
                .replace(FILENAME_TEMPLATE, filename)
                .replace(VERSION_TEMPLATE, &version.to_string())),
            DistroHook::Bin { bin, base_path } => {
                execute_binary(bin, base_path, Some(version.to_string()))
            }
        }
    }
}

/// Use the expected filename to determine the extension for this hook
///
/// This will include the multi-part `tar.gz` extension if it is present, otherwise it will use
/// the standard extension.
fn calculate_extension(filename: &str) -> Option<&str> {
    let mut parts = filename.rsplit('.');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(ext), Some("tar"), Some(_)) => {
            // .tar.gz style extension, return both parts
            //                          tar  .   gz
            let index = filename.len() - 3 - 1 - ext.len();
            filename.get(index..)
        }
        (Some(_), Some(""), None) => {
            // Dotfile, e.g. `.npmrc`, where the `.` character is at the beginning - No extension
            None
        }
        (Some(ext), Some(_), _) => {
            // Standard File Extension
            Some(ext)
        }
        _ => None,
    }
}

/// A hook for resolving the URL for metadata about a tool
#[derive(PartialEq, Eq, Debug)]
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
                .replace(OS_TEMPLATE, NODE_DISTRO_OS)
                .replace(FILENAME_TEMPLATE, filename)),
            MetadataHook::Bin { bin, base_path } => execute_binary(bin, base_path, None),
        }
    }
}

/// A hook for resolving the URL for the Yarn index
#[derive(PartialEq, Eq, Debug)]
pub struct YarnIndexHook {
    pub format: RegistryFormat,
    pub metadata: MetadataHook,
}

impl YarnIndexHook {
    /// Performs resolution of the metadata URL based on the given default file name
    pub fn resolve(&self, filename: &str) -> Fallible<String> {
        match &self.metadata {
            MetadataHook::Prefix(prefix) => Ok(format!("{}{}", prefix, filename)),
            MetadataHook::Template(template) => Ok(template
                .replace(ARCH_TEMPLATE, NODE_DISTRO_ARCH)
                .replace(OS_TEMPLATE, NODE_DISTRO_OS)
                .replace(FILENAME_TEMPLATE, filename)),
            MetadataHook::Bin { bin, base_path } => execute_binary(bin, base_path, None),
        }
    }
}

/// Execute a shell command and return the trimmed stdout from that command
fn execute_binary(bin: &str, base_path: &Path, extra_arg: Option<String>) -> Fallible<String> {
    let mut trimmed = bin.trim().to_string();
    let mut words = parse_posix(&mut trimmed);
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
    use super::{calculate_extension, DistroHook, MetadataHook};
    use crate::tool::{NODE_DISTRO_ARCH, NODE_DISTRO_OS};
    use node_semver::Version;

    #[test]
    fn test_distro_prefix_resolve() {
        let prefix = "http://localhost/node/distro/";
        let filename = "node.tar.gz";
        let hook = DistroHook::Prefix(prefix.to_string());
        let version = Version::parse("1.0.0").unwrap();

        assert_eq!(
            hook.resolve(&version, filename)
                .expect("Could not resolve URL"),
            format!("{}{}", prefix, filename)
        );
    }

    #[test]
    fn test_distro_template_resolve() {
        let hook = DistroHook::Template(
            "http://localhost/node/{{os}}/{{arch}}/{{version}}/{{ext}}/{{filename}}".to_string(),
        );
        let version = Version::parse("1.0.0").unwrap();

        // tar.gz format has extra handling, to support a multi-part extension
        let expected = format!(
            "http://localhost/node/{}/{}/{}/tar.gz/node-v1.0.0.tar.gz",
            NODE_DISTRO_OS, NODE_DISTRO_ARCH, version
        );
        assert_eq!(
            hook.resolve(&version, "node-v1.0.0.tar.gz")
                .expect("Could not resolve URL"),
            expected
        );

        // zip is a standard extension
        let expected = format!(
            "http://localhost/node/{}/{}/{}/zip/node-v1.0.0.zip",
            NODE_DISTRO_OS, NODE_DISTRO_ARCH, version
        );
        assert_eq!(
            hook.resolve(&version, "node-v1.0.0.zip")
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
        let hook = MetadataHook::Template(
            "http://localhost/node/{{os}}/{{arch}}/{{filename}}".to_string(),
        );
        let expected = format!(
            "http://localhost/node/{}/{}/index.json",
            NODE_DISTRO_OS, NODE_DISTRO_ARCH
        );

        assert_eq!(
            hook.resolve("index.json").expect("Could not resolve URL"),
            expected
        );
    }

    #[test]
    fn test_calculate_extension() {
        // Handles .tar.* files
        assert_eq!(calculate_extension("file.tar.gz"), Some("tar.gz"));
        assert_eq!(calculate_extension("file.tar.xz"), Some("tar.xz"));
        assert_eq!(calculate_extension("file.tar.xyz"), Some("tar.xyz"));

        // Handles dotfiles
        assert_eq!(calculate_extension(".filerc"), None);

        // Handles standard extensions
        assert_eq!(calculate_extension("tar.gz"), Some("gz"));
        assert_eq!(calculate_extension("file.zip"), Some("zip"));

        // Handles files with no extension at all
        assert_eq!(calculate_extension("bare_file"), None);
    }
}
