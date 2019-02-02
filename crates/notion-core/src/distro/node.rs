//! Provides the `NodeDistro` type, which represents a provisioned Node distribution.

use std::fs::{read_to_string, rename, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::string::ToString;

use serde::Deserialize;

use super::{Distro, Fetched};
use crate::distro::error::DownloadError;
use crate::distro::DistroVersion;
use crate::fs::ensure_containing_dir_exists;
use crate::inventory::NodeCollection;
use crate::path;
use crate::style::{progress_bar, Action};
use crate::tool::ToolSpec;
use crate::version::VersionSpec;
use archive::{self, Archive};
use tempfile::tempdir;

use notion_fail::{Fallible, ResultExt};
use semver::Version;

#[cfg(feature = "mock-network")]
use mockito;
use serde_json;

cfg_if::cfg_if! {
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
    let npm_version_file_path = path::node_npm_version_file(&node.to_string())?;
    Ok(read_to_string(npm_version_file_path)
        .unknown()?
        .parse()
        .unknown()?)
}

/// Save the default npm version to the filesystem for a given version of Node
fn save_default_npm_version(node: &Version, npm: &Version) -> Fallible<()> {
    let npm_version_file_path = path::node_npm_version_file(&node.to_string())?;
    let mut npm_version_file = File::create(npm_version_file_path).unknown()?;
    npm_version_file
        .write_all(npm.to_string().as_bytes())
        .unknown()?;
    Ok(())
}

/// Check if the fetched file is valid. It may have been corrupted or interrupted in the middle of
/// downloading.
// ISSUE(#134) - verify checksum
fn distro_is_valid(file: &PathBuf) -> bool {
    if file.is_file() {
        if let Ok(file) = File::open(file) {
            match archive::load_native(file) {
                Ok(_) => return true,
                Err(_) => return false,
            }
        }
    }
    false
}

#[derive(Deserialize)]
pub struct Manifest {
    version: String,
}

impl Manifest {
    fn read(path: &Path) -> Fallible<Manifest> {
        let file = File::open(path).unknown()?;
        serde_json::de::from_reader(file).unknown()
    }

    fn version(path: &Path) -> Fallible<Version> {
        Manifest::read(path)?.version.parse().unknown()
    }
}

impl Distro for NodeDistro {
    /// Provision a Node distribution from the public Node distributor (`https://nodejs.org`).
    fn public(version: Version) -> Fallible<Self> {
        let distro_file_name = path::node_distro_file_name(&version.to_string());
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
        let distro_file_name = path::node_distro_file_name(&version.to_string());
        let distro_file = path::node_inventory_dir()?.join(&distro_file_name);

        if distro_is_valid(&distro_file) {
            return NodeDistro::local(version, File::open(distro_file).unknown()?);
        }

        ensure_containing_dir_exists(&distro_file)?;
        Ok(NodeDistro {
            archive: archive::fetch_native(url, &distro_file).with_context(
                DownloadError::for_tool(
                    ToolSpec::Node(VersionSpec::exact(&version)),
                    url.to_string(),
                ),
            )?,
            version: version,
        })
    }

    /// Provision a Node distribution from the filesystem.
    fn local(version: Version, file: File) -> Fallible<Self> {
        Ok(NodeDistro {
            archive: archive::load_native(file).unknown()?,
            version: version,
        })
    }

    /// Produces a reference to this distribution's Node version.
    fn version(&self) -> &Version {
        &self.version
    }

    /// Fetches this version of Node. (It is left to the responsibility of the `NodeCollection`
    /// to update its state after fetching succeeds.)
    fn fetch(self, collection: &NodeCollection) -> Fallible<Fetched<DistroVersion>> {
        if collection.contains(&self.version) {
            let npm = load_default_npm_version(&self.version)?;

            return Ok(Fetched::Already(DistroVersion::Node(self.version, npm)));
        }

        let temp = tempdir().unknown()?;
        let bar = progress_bar(
            Action::Fetching,
            &format!("v{}", self.version),
            self.archive
                .uncompressed_size()
                .unwrap_or(self.archive.compressed_size()),
        );

        self.archive
            .unpack(temp.path(), &mut |_, read| {
                bar.inc(read as u64);
            })
            .unknown()?;

        let version_string = self.version.to_string();

        let npm_package_json = temp
            .path()
            .join(path::node_archive_npm_package_json_path(&version_string));

        let npm = Manifest::version(&npm_package_json)?;

        // Save the npm version number in the npm version file for this distro:
        save_default_npm_version(&self.version, &npm)?;

        let dest = path::node_image_dir(&version_string, &npm.to_string())?;

        ensure_containing_dir_exists(&dest)?;

        rename(
            temp.path()
                .join(path::node_archive_root_dir_name(&version_string)),
            dest,
        )
        .unknown()?;

        bar.finish_and_clear();
        Ok(Fetched::Now(DistroVersion::Node(self.version, npm)))
    }
}
