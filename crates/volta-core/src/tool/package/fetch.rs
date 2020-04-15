//! Provides fetcher for 3rd-party packages

use std::fs::{rename, write, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::error::ErrorDetails;
use crate::fs::{create_staging_dir, ensure_dir_does_not_exist, read_dir_eager, read_file};
use crate::layout::volta_home;
use crate::platform::CliPlatform;
use crate::run::{self, ToolCommand};
use crate::session::Session;
use crate::style::{progress_bar, progress_spinner, tool_version};
use crate::tool::PackageDetails;
use archive::{Archive, Tarball};
use fs_utils::ensure_containing_dir_exists;
use log::debug;
use semver::Version;
use sha1::{Digest, Sha1};
use volta_fail::{throw, Fallible, ResultExt};

pub fn fetch(name: &str, details: &PackageDetails, session: &mut Session) -> Fallible<()> {
    let version_string = details.version.to_string();
    let home = volta_home()?;
    let cache_file = home.package_distro_file(&name, &version_string);
    let shasum_file = home.package_distro_shasum(&name, &version_string);

    let (archive, cached) = match load_cached_distro(&cache_file, &shasum_file) {
        Some(archive) => {
            debug!(
                "Loading {} from cached archive at '{}'",
                tool_version(&name, &version_string),
                cache_file.display(),
            );
            (archive, true)
        }
        None => {
            let archive = fetch_remote_distro(&cache_file, &name, &details, session)?;
            (archive, false)
        }
    };

    unpack_archive(archive, name, &details.version)?;

    if cached {
        Ok(())
    } else {
        // Save the shasum in a file
        write(&shasum_file, details.shasum.as_bytes()).with_context(|_| {
            ErrorDetails::WritePackageShasumError {
                package: name.into(),
                version: version_string,
                file: shasum_file,
            }
        })
    }
}

fn load_cached_distro(file: &Path, shasum_file: &Path) -> Option<Box<dyn Archive>> {
    let mut distro = File::open(file).ok()?;
    let stored_shasum = read_file(shasum_file).ok()??; // `??`: Err(_) *or* Ok(None) -> None

    let mut buffer = Vec::new();
    distro.read_to_end(&mut buffer).ok()?;

    // calculate the shasum
    let mut hasher = Sha1::new();
    hasher.input(buffer);
    let result = hasher.result();
    let calculated_shasum = hex::encode(&result);

    if stored_shasum != calculated_shasum {
        return None;
    }

    distro.seek(SeekFrom::Start(0)).ok()?;
    Tarball::load(distro).ok()
}

fn fetch_remote_distro(
    path: &Path,
    name: &str,
    details: &PackageDetails,
    session: &mut Session,
) -> Fallible<Box<dyn Archive>> {
    ensure_containing_dir_exists(&path).with_context(|_| ErrorDetails::ContainingDirError {
        path: path.to_path_buf(),
    })?;

    // path.parent() will always be Some, because the previous call to ensure_containing_dir_exists would
    // error otherwise
    let dir = path.parent().unwrap();

    let command = npm_pack_command_for(name, &details.version.to_string()[..], session, dir)?;
    debug!("Running command: `{:?}`", command);

    debug!(
        "Downloading {} via npm pack to {}",
        tool_version(name, details.version.to_string()),
        dir.to_str().unwrap()
    );
    let spinner = progress_spinner(&format!(
        "Downloading {}",
        tool_version(name, details.version.to_string()),
    ));
    let output = command.output()?;
    spinner.finish_and_clear();

    if !output.status.success() {
        debug!(
            "Command failed, stderr is:\n{}",
            String::from_utf8_lossy(&output.stderr).to_string()
        );
        debug!("Exit code is {:?}", output.status.code());
        throw!(ErrorDetails::NpmPackFetchError {
            package: tool_version(name, details.version.to_string()),
        });
    }

    let filename = String::from_utf8_lossy(&output.stdout);
    // The output from `npm pack` contains a newline, so we'll trim it here.
    let trimmed_filename = filename.trim();

    if trimmed_filename.is_empty() {
        throw!(ErrorDetails::NpmPackUnpackError {
            package: tool_version(name, details.version.to_string())
        });
    }

    let tarball_from_npm_pack = dir.join(trimmed_filename.to_string());

    if !tarball_from_npm_pack.exists() {
        throw!(ErrorDetails::NpmPackUnpackError {
            package: tool_version(name, details.version.to_string())
        });
    }

    // If `npm pack` didn't name the tarball what we expect (usually because of scoped packages),
    // move it to where we expect it to be.
    if tarball_from_npm_pack != path {
        debug!(
            "Moving the tarball from {:?} to the expected path {:?}",
            tarball_from_npm_pack, path
        );
        rename(tarball_from_npm_pack, path).with_context(|_| ErrorDetails::NpmPackUnpackError {
            package: tool_version(name, details.version.to_string()),
        })?;
    }

    debug!("Attempting to load {:?}", path);
    let distro = File::open(path).with_context(|_| ErrorDetails::NpmPackUnpackError {
        package: tool_version(name, details.version.to_string()),
    })?;

    Tarball::load(distro).with_context(|_| ErrorDetails::NpmPackUnpackError {
        package: tool_version(name, details.version.to_string()),
    })
}

// build a command to run `npm pack`
fn npm_pack_command_for(
    name: &str,
    version: &str,
    session: &mut Session,
    current_dir: &Path,
) -> Fallible<ToolCommand> {
    let mut command = run::npm::command(CliPlatform::default(), session)?;
    command.arg("pack");
    command.arg("--no-update-notifier");
    command.arg(format!("{}@{}", name, version));
    command.current_dir(current_dir);
    Ok(command)
}

fn unpack_archive(archive: Box<dyn Archive>, name: &str, version: &Version) -> Fallible<()> {
    let temp = create_staging_dir()?;
    debug!("Unpacking {} into '{}'", name, temp.path().display());

    let progress = progress_bar(
        archive.origin(),
        &tool_version(&name, &version),
        archive
            .uncompressed_size()
            .unwrap_or_else(|| archive.compressed_size()),
    );

    archive
        .unpack(temp.path(), &mut |_, read| {
            progress.inc(read as u64);
        })
        .with_context(|_| ErrorDetails::UnpackArchiveError {
            tool: name.into(),
            version: version.to_string(),
        })?;

    let image_dir = volta_home()?.package_image_dir(&name, &version.to_string());
    // ensure that the dir where this will be unpacked exists
    ensure_containing_dir_exists(&image_dir).with_context(|_| {
        ErrorDetails::ContainingDirError {
            path: image_dir.clone(),
        }
    })?;
    // and ensure that the target directory does not exist
    ensure_dir_does_not_exist(&image_dir)?;

    let unpack_dir = find_unpack_dir(temp.path())?;
    rename(unpack_dir, &image_dir).with_context(|_| ErrorDetails::SetupToolImageError {
        tool: name.into(),
        version: version.to_string(),
        dir: image_dir.clone(),
    })?;

    progress.finish_and_clear();

    // Note: We write this after the progress bar is finished to avoid display bugs with re-renders of the progress
    debug!("Installing {} in '{}'", name, image_dir.display());

    Ok(())
}

/// Figure out the unpacked package directory name dynamically
///
/// Packages typically extract to a "package" directory, but not always
fn find_unpack_dir(in_dir: &Path) -> Fallible<PathBuf> {
    let dirs: Vec<_> = read_dir_eager(in_dir)
        .with_context(|_| ErrorDetails::PackageUnpackError)?
        .collect();

    // if there is only one directory, return that
    if let [(entry, metadata)] = dirs.as_slice() {
        if metadata.is_dir() {
            return Ok(entry.path());
        }
    }
    // there is more than just a single directory here, something is wrong
    Err(ErrorDetails::PackageUnpackError.into())
}
