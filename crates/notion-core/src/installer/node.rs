//! Provides the `Installer` type, which represents a provisioned Node installer.

use std::fs::{rename, File};
use std::string::ToString;

use super::Installed;
use catalog::NodeCatalog;
use node_archive::{self, Archive};
use path;
use style::{progress_bar, Action};

use notion_fail::{Fallible, ResultExt};
use semver::Version;

const PUBLIC_NODE_SERVER_ROOT: &'static str = "https://nodejs.org/dist/";

/// A provisioned Node installer.
pub struct Installer {
    archive: Box<Archive>,
    version: Version,
}

impl Installer {
    /// Provision an `Installer` from the public Node distributor (`https://nodejs.org`).
    pub fn public(version: Version) -> Fallible<Self> {
        let archive_file = path::archive_file(&version.to_string());
        let url = format!("{}v{}/{}", PUBLIC_NODE_SERVER_ROOT, version, &archive_file);
        Installer::remote(version, &url)
    }

    /// Provision an `Installer` from a remote distributor.
    pub fn remote(version: Version, url: &str) -> Fallible<Self> {
        let archive_file = path::archive_file(&version.to_string());
        let cache_file = path::node_cache_dir()?.join(&archive_file);

        if cache_file.is_file() {
            return Installer::cached(version, File::open(cache_file).unknown()?);
        }

        Ok(Installer {
            archive: node_archive::fetch(url, &cache_file).unknown()?,
            version: version,
        })
    }

    /// Provision an `Installer` from the filesystem.
    pub fn cached(version: Version, file: File) -> Fallible<Self> {
        Ok(Installer {
            archive: node_archive::load(file).unknown()?,
            version: version,
        })
    }

    /// Produces a reference to this installer's Node version.
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Installs this version of Node. (It is left to the responsibility of the `NodeCatalog`
    /// to update its state after installation succeeds.)
    pub fn install(self, catalog: &NodeCatalog) -> Fallible<Installed> {
        if catalog.contains(&self.version) {
            return Ok(Installed::Already(self.version));
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
            dest.join(path::archive_root_dir(&version_string)),
            path::node_version_dir(&version_string)?,
        ).unknown()?;

        bar.finish_and_clear();
        Ok(Installed::Now(self.version))
    }
}
