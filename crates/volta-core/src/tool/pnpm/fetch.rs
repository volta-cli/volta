//! Provides fetcher for npm distributions

use std::fs::{write, File};
use std::path::{Path, PathBuf};

use super::super::download_tool_error;
use super::super::registry::public_registry_package;
use crate::error::{Context, ErrorKind, Fallible};
use crate::fs::{create_staging_dir, create_staging_file, rename, set_executable};
use crate::hook::ToolHooks;
use crate::layout::volta_home;
use crate::style::{progress_bar, tool_version};
use crate::tool::{self, Pnpm};
use crate::version::VersionSpec;
use archive::{Archive, Tarball};
use fs_utils::ensure_containing_dir_exists;
use log::debug;
use semver::Version;

pub fn fetch(version: &Version, hooks: Option<&ToolHooks<Pnpm>>) -> Fallible<()> {
    let pnpm_dir = volta_home()?.pnpm_inventory_dir();
    let cache_file = pnpm_dir.join(Pnpm::archive_filename(&version.to_string()));

    let (archive, staging) = match load_cached_distro(&cache_file) {
        Some(archive) => {
            debug!(
                "Loading {} from cached archive at '{}'",
                tool_version("pnpm", &version),
                cache_file.display()
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
                tool: "pnpm".into(),
            })?;
    }

    Ok(())
}

/// Unpack the pnpm archive into the image directory so that it is ready for use
fn unpack_archive(archive: Box<dyn Archive>, version: &Version) -> Fallible<()> {
    let temp = create_staging_dir()?;
    debug!("Unpacking pnpm into '{}'", temp.path().display());

    let progress = progress_bar(
        archive.origin(),
        &tool_version("pnpm", version),
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
            tool: "pnpm".into(),
            version: version_string.clone(),
        })?;

    let bin_path = temp.path().join("package").join("bin");
    create_launcher(&bin_path, "pnpm")?;
    create_launcher(&bin_path, "pnpx")?;

    #[cfg(windows)]
    {
        create_cmd_launcher(&bin_path, "pnpm")?;
        create_cmd_launcher(&bin_path, "pnpx")?;
    }

    let dest = volta_home()?.pnpm_image_dir(&version_string);
    ensure_containing_dir_exists(&dest)
        .with_context(|| ErrorKind::ContainingDirError { path: dest.clone() })?;

    rename(temp.path().join("package"), &dest).with_context(|| ErrorKind::SetupToolImageError {
        tool: "pnpm".into(),
        version: version_string.clone(),
        dir: dest.clone(),
    })?;

    progress.finish_and_clear();

    // Note: We write this after the progress bar is finished to avoid display bugs with re-renders of the progress
    debug!("Installing pnpm in '{}'", dest.display());

    Ok(())
}

/// Return the archive if it is valid. It may have been corrupted or interrupted in the middle of
/// downloading.
/// ISSUE(#134) - verify checksum
fn load_cached_distro(file: &PathBuf) -> Option<Box<dyn Archive>> {
    let file = File::open(file).ok()?;
    Tarball::load(file).ok()
}

/// Determine the remote URL to download from, using the hooks if avaialble
fn determine_remote_url(version: &Version, hooks: Option<&ToolHooks<Pnpm>>) -> Fallible<String> {
    let version_str = version.to_string();
    match hooks {
        Some(&ToolHooks {
            distro: Some(ref hook),
            ..
        }) => {
            debug!("Using pnpm.distro hook to determine download URL");
            let distro_file_name = Pnpm::archive_filename(&version_str);
            hook.resolve(&version, &distro_file_name)
        }
        _ => Ok(public_registry_package("pnpm", &version_str)),
    }
}

/// Fetch the distro archive from the internet
fn fetch_remote_distro(
    version: &Version,
    url: &str,
    staging_path: &Path,
) -> Fallible<Box<dyn Archive>> {
    debug!("Downloading {} from {}", tool_version("pnpm", version), url);
    Tarball::fetch(url, staging_path).with_context(download_tool_error(
        tool::Spec::Pnpm(VersionSpec::Exact(version.clone())),
        url,
    ))
}

/// Create the launcher script
fn create_launcher(base_path: &Path, tool: &str) -> Fallible<()> {
    let path = base_path.join(tool);
    write(
        &path,
        // Note: Adapted from the existing npm/npx launcher, without unnecessary detection of Node location
        format!(
            r#"#!/bin/sh
(set -o igncr) 2>/dev/null && set -o igncr; # cygwin encoding fix

basedir=`dirname "$0"`

case `uname` in
    *CYGWIN*) basedir=`cygpath -w "$basedir"`;;
esac

node "$basedir/{}.js" "$@"
"#,
            tool
        ),
    )
    .and_then(|_| set_executable(&path))
    .with_context(|| ErrorKind::WriteLauncherError { tool: tool.into() })
}

/// Create the CMD launcher
#[cfg(windows)]
fn create_cmd_launcher(base_path: &Path, tool: &str) -> Fallible<()> {
    write(
        base_path.join(format!("{}.cmd", tool)),
        // Note: Adapted from the existing npm/npx cmd launcher, without unnecessary detection of Node location
        format!(
            r#"@ECHO OFF

node "%~dp0\{}.js" %*
"#,
            tool
        ),
    )
    .with_context(|| ErrorKind::WriteLauncherError { tool: tool.into() })
}
