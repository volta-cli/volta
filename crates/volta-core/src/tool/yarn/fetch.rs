//! Provides fetcher for Yarn distributions

use std::fs::File;
use std::path::{Path, PathBuf};

use super::super::download_tool_error;
use super::super::registry::public_registry_package;
use crate::error::{Context, ErrorKind, Fallible};
use crate::fs::{create_staging_dir, create_staging_file, rename};
use crate::hook::ToolHooks;
use crate::layout::volta_home;
use crate::style::{progress_bar, tool_version};
use crate::tool::{self, Yarn};
use crate::version::VersionSpec;
use archive::{Archive, Tarball};
use fs_utils::ensure_containing_dir_exists;
use log::debug;
use semver::Version;

pub fn fetch(version: &Version, hooks: Option<&ToolHooks<Yarn>>) -> Fallible<()> {
    let yarn_dir = volta_home()?.yarn_inventory_dir();
    let cache_file = yarn_dir.join(Yarn::archive_filename(&version.to_string()));

    let (archive, staging) = match load_cached_distro(&cache_file) {
        Some(archive) => {
            debug!(
                "Loading {} from cached archive at '{}'",
                tool_version("yarn", &version),
                cache_file.display(),
            );
            (archive, None)
        }
        None => {
            let staging = create_staging_file()?;
            let remote_url = determine_remote_url(&version, hooks)?;
            let archive = fetch_remote_distro(&version, &remote_url, staging.path())?;
            (archive, Some(staging))
        }
    };

    unpack_archive(archive, version)?;

    if let Some(staging_file) = staging {
        ensure_containing_dir_exists(&cache_file).with_context(|| {
            ErrorKind::ContainingDirError {
                path: cache_file.clone(),
            }
        })?;
        staging_file
            .persist(cache_file)
            .with_context(|| ErrorKind::PersistInventoryError {
                tool: "Yarn".into(),
            })?;
    }

    Ok(())
}

/// Unpack the yarn archive into the image directory so that it is ready for use
fn unpack_archive(archive: Box<dyn Archive>, version: &Version) -> Fallible<()> {
    let temp = create_staging_dir()?;
    debug!("Unpacking yarn into '{}'", temp.path().display());

    let progress = progress_bar(
        archive.origin(),
        &tool_version("yarn", version),
        archive
            .uncompressed_size()
            .unwrap_or_else(|| archive.compressed_size()),
    );
    let version_string = version.to_string();

    archive
        .unpack(temp.path(), &mut |_, read| {
            progress.inc(read as u64);
        })
        .with_context(|| ErrorKind::UnpackArchiveError {
            tool: "Yarn".into(),
            version: version_string.clone(),
        })?;

    let dest = volta_home()?.yarn_image_dir(&version_string);
    ensure_containing_dir_exists(&dest)
        .with_context(|| ErrorKind::ContainingDirError { path: dest.clone() })?;

    rename(
        temp.path().join(Yarn::archive_basename(&version_string)),
        &dest,
    )
    .with_context(|| ErrorKind::SetupToolImageError {
        tool: "Yarn".into(),
        version: version_string.clone(),
        dir: dest.clone(),
    })?;

    progress.finish_and_clear();

    // Note: We write this after the progress bar is finished to avoid display bugs with re-renders of the progress
    debug!("Installing yarn in '{}'", dest.display());

    Ok(())
}

/// Return the archive if it is valid. It may have been corrupted or interrupted in the middle of
/// downloading.
// ISSUE(#134) - verify checksum
fn load_cached_distro(file: &PathBuf) -> Option<Box<dyn Archive>> {
    if file.is_file() {
        let file = File::open(file).ok()?;
        Tarball::load(file).ok()
    } else {
        None
    }
}

/// Determine the remote URL to download from, using the hooks if available
fn determine_remote_url(version: &Version, hooks: Option<&ToolHooks<Yarn>>) -> Fallible<String> {
    let version_str = version.to_string();
    match hooks {
        Some(&ToolHooks {
            distro: Some(ref hook),
            ..
        }) => {
            debug!("Using yarn.distro hook to determine download URL");
            let distro_file_name = Yarn::archive_filename(&version_str);
            hook.resolve(&version, &distro_file_name)
        }
        _ => Ok(public_registry_package("yarn", &version_str)),
    }
}

/// Fetch the distro archive from the internet
fn fetch_remote_distro(
    version: &Version,
    url: &str,
    staging_path: &Path,
) -> Fallible<Box<dyn Archive>> {
    debug!("Downloading {} from {}", tool_version("yarn", version), url);
    Tarball::fetch(url, staging_path).with_context(download_tool_error(
        tool::Spec::Yarn(VersionSpec::Exact(version.clone())),
        url,
    ))
}
