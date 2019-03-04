//! Provides the `YarnDistro` type, which represents a provisioned Yarn distribution.

use std::fs::{rename, File};
use std::path::PathBuf;
use std::string::ToString;

use semver::Version;
use tempfile::tempdir_in;

use archive::{Archive, Tarball};
use notion_fail::{Fallible, ResultExt};

use super::{download_tool_error, Distro, Fetched};
use crate::fs::ensure_containing_dir_exists;
use crate::hook::ToolHooks;
use crate::inventory::YarnCollection;
use crate::path;
use crate::style::progress_bar;
use crate::tool::ToolSpec;
use crate::version::VersionSpec;

#[cfg(feature = "mock-network")]
use mockito;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-network")] {
        fn public_yarn_server_root() -> String {
            mockito::SERVER_URL.to_string()
        }
    } else {
        fn public_yarn_server_root() -> String {
            "https://github.com/yarnpkg/yarn/releases/download".to_string()
        }
    }
}

/// A provisioned Yarn distribution.
pub struct YarnDistro {
    archive: Box<dyn Archive>,
    version: Version,
}

/// Check if the fetched file is valid. It may have been corrupted or interrupted in the middle of
/// downloading.
// ISSUE(#134) - verify checksum
fn distro_is_valid(file: &PathBuf) -> bool {
    if file.is_file() {
        if let Ok(file) = File::open(file) {
            match Tarball::load(file) {
                Ok(_) => return true,
                Err(_) => return false,
            }
        }
    }
    false
}

impl YarnDistro {
    /// Provision a Yarn distribution from the public distributor (`https://yarnpkg.com`).
    fn public(version: Version) -> Fallible<Self> {
        let version_str = version.to_string();
        let distro_file_name = path::yarn_distro_file_name(&version_str);
        let url = format!(
            "{}/v{}/{}",
            public_yarn_server_root(),
            version_str,
            distro_file_name
        );
        YarnDistro::remote(version, &url)
    }

    /// Provision a Yarn distribution from a remote distributor.
    fn remote(version: Version, url: &str) -> Fallible<Self> {
        let distro_file_name = path::yarn_distro_file_name(&version.to_string());
        let distro_file = path::yarn_inventory_dir()?.join(&distro_file_name);

        if distro_is_valid(&distro_file) {
            return YarnDistro::local(version, File::open(distro_file).unknown()?);
        }

        ensure_containing_dir_exists(&distro_file)?;
        Ok(YarnDistro {
            archive: Tarball::fetch(url, &distro_file).with_context(download_tool_error(
                ToolSpec::Yarn(VersionSpec::exact(&version)),
                url,
            ))?,
            version: version,
        })
    }

    /// Provision a Yarn distribution from the filesystem.
    fn local(version: Version, file: File) -> Fallible<Self> {
        Ok(YarnDistro {
            archive: Tarball::load(file).unknown()?,
            version: version,
        })
    }
}

impl Distro for YarnDistro {
    type VersionDetails = Version;
    type ResolvedVersion = Version;

    /// Provisions a new Distro based on the Version and possible Hooks
    fn new(
        _name: String,
        version: Self::ResolvedVersion,
        hooks: Option<&ToolHooks<Self>>,
    ) -> Fallible<Self> {
        match hooks {
            Some(&ToolHooks {
                distro: Some(ref hook),
                ..
            }) => {
                let url =
                    hook.resolve(&version, &path::yarn_distro_file_name(&version.to_string()))?;
                YarnDistro::remote(version, &url)
            }
            _ => YarnDistro::public(version),
        }
    }

    /// Produces a reference to this distro's Yarn version.
    fn version(&self) -> &Version {
        &self.version
    }

    /// Fetches this version of Yarn. (It is left to the responsibility of the `YarnCollection`
    /// to update its state after fetching succeeds.)
    fn fetch(self, collection: &YarnCollection) -> Fallible<Fetched<Version>> {
        if collection.contains(&self.version) {
            return Ok(Fetched::Already(self.version));
        }

        let temp = tempdir_in(path::tmp_dir()?).unknown()?;
        let bar = progress_bar(
            self.archive.action(),
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

        let dest = path::yarn_image_dir(&version_string)?;

        ensure_containing_dir_exists(&dest)?;

        rename(
            temp.path()
                .join(path::yarn_archive_root_dir_name(&version_string)),
            dest,
        )
        .unknown()?;

        bar.finish_and_clear();
        Ok(Fetched::Now(self.version))
    }
}
