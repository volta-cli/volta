//! Provides the `Installer` type, which represents a provisioned Node installer.

use std::fs::{rename, File};
use std::string::ToString;

use super::{Distro, Fetched};
use catalog::NodeCollection;
use distro::error::DownloadError;
use node_archive::{self, Archive};
use path;
use style::{progress_bar, Action};

use notion_fail::{Fallible, ResultExt};
use semver::Version;

const PUBLIC_NODE_SERVER_ROOT: &'static str = "https://nodejs.org/dist/";

/// A provisioned Node distribution.
pub struct NodeDistro {
    archive: Box<Archive>,
    version: Version,
}

impl Distro for NodeDistro {
    /// Provision an `Installer` from the public Node distributor (`https://nodejs.org`).
    fn public(version: Version) -> Fallible<Self> {
        let archive_file = path::node_archive_file(&version.to_string());
        let url = format!("{}v{}/{}", PUBLIC_NODE_SERVER_ROOT, version, &archive_file);
        NodeDistro::remote(version, &url)
    }

    /// Provision an `Installer` from a remote distributor.
    fn remote(version: Version, url: &str) -> Fallible<Self> {
        let archive_file = path::node_archive_file(&version.to_string());
        let cache_file = path::node_cache_dir()?.join(&archive_file);

        if cache_file.is_file() {
            return NodeDistro::cached(version, File::open(cache_file).unknown()?);
        }

        Ok(NodeDistro {
            archive: node_archive::fetch(url, &cache_file)
                .with_context(DownloadError::for_version(version.to_string()))?,
            version: version,
        })
    }

    /// Provision an `Installer` from the filesystem.
    fn cached(version: Version, file: File) -> Fallible<Self> {
        Ok(NodeDistro {
            archive: node_archive::load(file).unknown()?,
            version: version,
        })
    }

    /// Produces a reference to this installer's Node version.
    fn version(&self) -> &Version {
        &self.version
    }

    /// Fetches this version of Node. (It is left to the responsibility of the `NodeCollection`
    /// to update its state after fetching succeeds.)
    fn fetch(self, collection: &NodeCollection) -> Fallible<Fetched> {
        if collection.contains(&self.version) {
            return Ok(Fetched::Already(self.version));
        }

        let dest = path::node_versions_dir()?;
        let bar = progress_bar(
            Action::Installing,
            &format!("v{}", self.version),
            self.archive
                .uncompressed_size()
                .unwrap_or(self.archive.compressed_size()),
        );

        self.archive
            .unpack(&dest, &mut |_, read| {
                bar.inc(read as u64);
            })
            .unknown()?;

        let version_string = self.version.to_string();
        rename(
            dest.join(path::node_archive_root_dir(&version_string)),
            path::node_version_dir(&version_string)?,
        ).unknown()?;

        bar.finish_and_clear();
        Ok(Fetched::Now(self.version))
    }
}
