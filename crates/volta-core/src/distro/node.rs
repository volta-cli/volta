//! Provides the `NodeDistro` type, which represents a provisioned Node distribution.

use std::fs::{read_to_string, rename, write, File};
use std::path::{Path, PathBuf};
use std::string::ToString;

use archive::{self, Archive};
use serde::Deserialize;
use tempfile::tempdir_in;

use super::{download_tool_error, Distro, Fetched};
use crate::error::ErrorDetails;
use crate::fs::ensure_containing_dir_exists;
use crate::hook::ToolHooks;
use crate::inventory::NodeCollection;
use crate::layout::layout;
use crate::style::{progress_bar, tool_version};
use crate::tool::ToolSpec;
use crate::version::VersionSpec;

use cfg_if::cfg_if;
use log::debug;
use semver::Version;
use volta_fail::{Fallible, ResultExt};

#[cfg(feature = "mock-network")]
use mockito;
use serde_json;

cfg_if! {
    if #[cfg(feature = "mock-network")] {
        fn public_node_server_root() -> String {
            mockito::SERVER_URL.to_string()
        }
    } else {
        fn public_node_server_root() -> String {
            "https://nodejs.org/dist".to_string()
        }
    }
}

// These are taken from: https://nodejs.org/dist/index.json and are used
// by `NodeDistro::basename` to determine the name and internal layout of
// a Node distribution archive.

cfg_if! {
    if #[cfg(target_os = "windows")] {
        /// The OS component of a Node distro's filename.
        pub const OS: &'static str = "win";
    } else if #[cfg(target_os = "macos")] {
        /// The OS component of a Node distro's filename.
        pub const OS: &'static str = "darwin";
    } else if #[cfg(target_os = "linux")] {
        /// The OS component of a Node distro's filename.
        pub const OS: &'static str = "linux";
    } else {
        compile_error!("Unsupported operating system (expected Windows, macOS, or Linux).");
    }
}

cfg_if! {
    if #[cfg(target_arch = "x86")] {
        /// The system architecture component of a Node distro's name.
        pub const ARCH: &'static str = "x86";
    } else if #[cfg(target_arch = "x86_64")] {
        /// The system architecture component of a Node distro's name.
        pub const ARCH: &'static str = "x64";
    } else {
        compile_error!("Unsupported target_arch variant (expected 'x86' or 'x64').");
    }
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        /// Filename extension for Node distro files.
        pub const NODE_DISTRO_EXTENSION: &'static str = "zip";
    } else {
        /// Filename extension for Node distro files.
        pub const NODE_DISTRO_EXTENSION: &'static str = "tar.gz";
    }
}

/// A provisioned Node distribution.
pub struct NodeDistro {
    archive: Box<dyn Archive>,
    version: Version,
}

/// A full Node version including not just the version of Node itself
/// but also the specific version of npm installed globally with that
/// Node installation.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct NodeVersion {
    /// The version of Node itself.
    pub runtime: Version,
    /// The npm version globally installed with the Node distro.
    pub npm: Version,
}

/// Load the local npm version file to determine the default npm version for a given version of Node
pub fn load_default_npm_version(node: &Version) -> Fallible<Version> {
    let npm_version_file_path = layout()?.user.node_npm_version_file(&node.to_string());
    let npm_version = read_to_string(&npm_version_file_path).with_context(|_| {
        ErrorDetails::ReadDefaultNpmError {
            file: npm_version_file_path,
        }
    })?;
    VersionSpec::parse_version(npm_version)
}

/// Save the default npm version to the filesystem for a given version of Node
fn save_default_npm_version(node: &Version, npm: &Version) -> Fallible<()> {
    let npm_version_file_path = layout()?.user.node_npm_version_file(&node.to_string());
    write(&npm_version_file_path, npm.to_string().as_bytes()).with_context(|_| {
        ErrorDetails::WriteDefaultNpmError {
            file: npm_version_file_path,
        }
    })
}

/// Return the archive if it is valid. It may have been corrupted or interrupted in the middle of
/// downloading.
// ISSUE(#134) - verify checksum
fn load_cached_distro(file: &PathBuf) -> Option<Box<dyn Archive>> {
    if file.is_file() {
        if let Ok(file) = File::open(file) {
            if let Ok(archive) = archive::load_native(file) {
                return Some(archive);
            }
        }
    }
    None
}

#[derive(Deserialize)]
pub struct Manifest {
    version: String,
}

impl Manifest {
    fn read(path: &Path) -> Fallible<Manifest> {
        let file = File::open(path).with_context(|_| ErrorDetails::ReadNpmManifestError)?;
        serde_json::de::from_reader(file).with_context(|_| ErrorDetails::ParseNpmManifestError)
    }

    fn version(path: &Path) -> Fallible<Version> {
        VersionSpec::parse_version(Manifest::read(path)?.version)
    }
}

impl NodeDistro {
    pub fn basename(version: &str) -> String {
        format!("node-v{}-{}-{}", &version, OS, ARCH)
    }

    pub fn filename(version: &str) -> String {
        format!(
            "{}.{}",
            NodeDistro::basename(version),
            NODE_DISTRO_EXTENSION
        )
    }

    fn npm_manifest_entry(version: &str) -> PathBuf {
        Path::new(&NodeDistro::basename(version))
            .join("lib")
            .join("node_modules")
            .join("npm")
            .join("package.json")
    }

    /// Provision a Node distribution from the public Node distributor (`https://nodejs.org`).
    fn public(version: Version) -> Fallible<Self> {
        let distro_file_name = NodeDistro::filename(&version.to_string());
        let url = format!(
            "{}/v{}/{}",
            public_node_server_root(),
            version,
            &distro_file_name
        );
        NodeDistro::remote(version, &url)
    }

    /// Provision a Node distribution from a remote distributor.
    fn remote(version: Version, url: &str) -> Fallible<Self> {
        let distro_file_name = NodeDistro::filename(&version.to_string());
        let distro_file = layout()?.user.node_inventory_dir().join(&distro_file_name);

        if let Some(archive) = load_cached_distro(&distro_file) {
            debug!(
                "Loading node@{} from cached archive at {}",
                version,
                distro_file.display()
            );
            return Ok(NodeDistro { archive, version });
        }

        ensure_containing_dir_exists(&distro_file)?;
        debug!("Downloading node@{} from {}", version, url);

        Ok(NodeDistro {
            archive: archive::fetch_native(url, &distro_file).with_context(download_tool_error(
                ToolSpec::Node(VersionSpec::exact(&version)),
                url,
            ))?,
            version: version,
        })
    }
}

impl Distro for NodeDistro {
    type VersionDetails = NodeVersion;
    type ResolvedVersion = Version;

    /// Provisions a new Distro based on the Version and possible Hooks
    fn new(
        _name: &str,
        version: Self::ResolvedVersion,
        hooks: Option<&ToolHooks<Self>>,
    ) -> Fallible<Self> {
        match hooks {
            Some(&ToolHooks {
                distro: Some(ref hook),
                ..
            }) => {
                debug!("Using node.distro hook to determine download URL");
                let url = hook.resolve(&version, &NodeDistro::filename(&version.to_string()))?;
                NodeDistro::remote(version, &url)
            }
            _ => NodeDistro::public(version),
        }
    }

    /// Produces a reference to this distribution's Node version.
    fn version(&self) -> &Version {
        &self.version
    }

    /// Fetches this version of Node. (It is left to the responsibility of the `NodeCollection`
    /// to update its state after fetching succeeds.)
    fn fetch(self, collection: &NodeCollection) -> Fallible<Fetched<NodeVersion>> {
        if collection.contains(&self.version) {
            let npm = load_default_npm_version(&self.version)?;

            debug!(
                "node@{} has already been fetched, skipping install",
                &self.version
            );
            return Ok(Fetched::Already(NodeVersion {
                runtime: self.version,
                npm,
            }));
        }

        let layout = layout()?;

        let tmp_root = layout.user.tmp_dir();
        let temp = tempdir_in(&tmp_root).with_context(|_| ErrorDetails::CreateTempDirError {
            in_dir: tmp_root.to_path_buf(),
        })?;
        debug!("Unpacking node into {}", temp.path().display());

        let bar = progress_bar(
            self.archive.origin(),
            &tool_version("node", &self.version),
            self.archive
                .uncompressed_size()
                .unwrap_or(self.archive.compressed_size()),
        );

        let version_string = self.version.to_string();

        self.archive
            .unpack(temp.path(), &mut |_, read| {
                bar.inc(read as u64);
            })
            .with_context(|_| ErrorDetails::UnpackArchiveError {
                tool: String::from("Node"),
                version: version_string.clone(),
            })?;

        let version_string = self.version.to_string();
        let root_dir_name = NodeDistro::basename(&version_string);

        let npm_package_json = temp
            .path()
            .join(NodeDistro::npm_manifest_entry(&version_string));

        let npm = Manifest::version(&npm_package_json)?;

        // Save the npm version number in the npm version file for this distro:
        save_default_npm_version(&self.version, &npm)?;

        let dest = layout
            .user
            .node_image_dir(&version_string, &npm.to_string());

        ensure_containing_dir_exists(&dest)?;

        rename(temp.path().join(root_dir_name), &dest).with_context(|_| {
            ErrorDetails::SetupToolImageError {
                tool: String::from("Node"),
                version: version_string.clone(),
                dir: dest.clone(),
            }
        })?;

        bar.finish_and_clear();

        // Note: We write these after the progress bar is finished to avoid display bugs with re-renders of the progress
        debug!("Saving bundled npm version ({})", npm);
        debug!("Installing node in {}", dest.display());
        Ok(Fetched::Now(NodeVersion {
            runtime: self.version,
            npm,
        }))
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    fn test_filename() {
        assert_eq!(
            NodeDistro::filename("1.2.3"),
            format!("node-v1.2.3-{}-{}.{}", OS, ARCH, NODE_DISTRO_EXTENSION)
        );
    }

    #[test]
    fn test_basename() {
        assert_eq!(
            NodeDistro::basename("1.2.3"),
            format!("node-v1.2.3-{}-{}", OS, ARCH)
        );
    }

}
