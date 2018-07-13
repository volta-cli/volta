//! Provides the `Installer` type, which represents a provisioned Node installer.

use std::fs::{rename, File};
use std::string::ToString;

use super::{Install, Installed};
use catalog::YarnCollection;
use node_archive::{self, Archive};
use path;
use style::{progress_bar, Action};

use notion_fail::{Fallible, ResultExt};
use semver::Version;

const PUBLIC_YARN_SERVER_ROOT: &'static str =
    "https://github.com/notion-cli/yarn-releases/raw/master/dist/";

/// A provisioned Yarn installer.
pub struct YarnInstaller {
    archive: Box<Archive>,
    version: Version,
}

impl Install for YarnInstaller {
    /// Provision an `Installer` from the public Yarn distributor (`https://yarnpkg.com`).
    fn public(version: Version) -> Fallible<Self> {
        let archive_file = path::yarn_archive_file(&version.to_string());
        let url = format!("{}{}", PUBLIC_YARN_SERVER_ROOT, archive_file);
        YarnInstaller::remote(version, &url)
    }

    /// Provision an `Installer` from a remote distributor.
    fn remote(version: Version, url: &str) -> Fallible<Self> {
        let archive_file = path::yarn_archive_file(&version.to_string());
        let cache_file = path::yarn_cache_dir()?.join(&archive_file);

        if cache_file.is_file() {
            return YarnInstaller::cached(version, File::open(cache_file).unknown()?);
        }

        Ok(YarnInstaller {
            archive: node_archive::fetch(url, &cache_file).unknown()?,
            version: version,
        })
    }

    /// Provision an `Installer` from the filesystem.
    fn cached(version: Version, file: File) -> Fallible<Self> {
        Ok(YarnInstaller {
            archive: node_archive::load(file).unknown()?,
            version: version,
        })
    }

    /// Produces a reference to this installer's Yarn version.
    fn version(&self) -> &Version {
        &self.version
    }

    /// Installs this version of Yarn. (It is left to the responsibility of the `YarnCollection`
    /// to update its state after installation succeeds.)
    fn install(self, collection: &YarnCollection) -> Fallible<Installed> {
        if collection.contains(&self.version) {
            return Ok(Installed::Already(self.version));
        }

        let dest = path::yarn_versions_dir()?;
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
            dest.join(path::yarn_archive_root_dir(&version_string)),
            path::yarn_version_dir(&version_string)?,
        ).unknown()?;

        bar.finish_and_clear();
        Ok(Installed::Now(self.version))
    }
}
