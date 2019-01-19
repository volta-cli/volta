//! Provides the `YarnDistro` type, which represents a provisioned Yarn distribution.

use std::fs::{rename, File};
use std::path::PathBuf;
use std::string::ToString;

use super::{Distro, Fetched};
use archive::{Archive, Tarball};
use distro::error::DownloadError;
use distro::DistroVersion;
use fs::ensure_containing_dir_exists;
use hook::ToolHooks;
use inventory::YarnCollection;
use path;
use style::{progress_bar, Action};
use tool::ToolSpec;
use version::VersionSpec;

use notion_fail::{Fallible, ResultExt};
use semver::Version;

#[cfg(feature = "mock-network")]
use mockito;

cfg_if! {
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
    archive: Box<Archive>,
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
            archive: Tarball::fetch(url, &distro_file).with_context(DownloadError::for_tool(
                ToolSpec::Yarn(VersionSpec::exact(&version)),
                url.to_string(),
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

    /// Provisions a new Distro based on the Version and Possible Hooks
    fn new(version: Version, hooks: Option<&ToolHooks<Self>>) -> Fallible<Self> {
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
    fn fetch(self, collection: &YarnCollection) -> Fallible<Fetched<DistroVersion>> {
        if collection.contains(&self.version) {
            return Ok(Fetched::Already(DistroVersion::Yarn(self.version)));
        }

        let dest = path::yarn_image_root_dir()?;
        let bar = progress_bar(
            Action::Fetching,
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
            dest.join(path::yarn_archive_root_dir_name(&version_string)),
            path::yarn_image_dir(&version_string)?,
        )
        .unknown()?;

        bar.finish_and_clear();
        Ok(Fetched::Now(DistroVersion::Yarn(self.version)))
    }
}
