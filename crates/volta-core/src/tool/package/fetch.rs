//! Provides fetcher for 3rd-party packages

use std::ffi::OsString;
use std::fs::{rename, write, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use super::super::download_tool_error;
use crate::error::ErrorDetails;
use crate::fs::{create_staging_dir, ensure_dir_does_not_exist, read_dir_eager, read_file};
use crate::path;
use crate::run::{self, ToolCommand};
use crate::session::Session;
use crate::style::{progress_bar, progress_spinner, tool_version};
use crate::tool::{self, PackageDetails};
use crate::version::VersionSpec;
use archive::{Archive, Tarball};
use fs_utils::ensure_containing_dir_exists;
use log::debug;
use semver::Version;
use sha1::{Digest, Sha1};
use volta_fail::{throw, Fallible, ResultExt};

pub fn fetch(name: &str, details: &PackageDetails, session: &mut Session) -> Fallible<()> {
    let version_string = details.version.to_string();
    let cache_file = path::package_distro_file(&name, &version_string)?;
    let shasum_file = path::package_distro_shasum(&name, &version_string)?;

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
            let archive = fetch_remote_distro(
                tool::Spec::Package(name.into(), VersionSpec::exact(&details.version)),
                &details.tarball_url,
                &cache_file,
                &shasum_file,
                &name,
                &details,
                session,
            )?;
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
    spec: tool::Spec,
    url: &str,
    path: &Path,
    shasum_file: &Path,
    name: &str,
    details: &PackageDetails,
    session: &mut Session,
) -> Fallible<Box<Archive>> {
    debug!("YOU'RE IN MARK!!!");

    ensure_containing_dir_exists(&path);

    let dir = path.parent().unwrap();

    let command = npm_pack_command_for(name, &details.version.to_string()[..], session, Some(dir))?;
    debug!("Running command: `{:?}`", command);

    debug!(
        "Downloading via {} npm pack to {}",
        tool_version(name, details.version.to_string()),
        dir.to_str().unwrap()
    );
    let spinner = progress_spinner(&format!(
        "Downloading via {} npm pack to {}",
        tool_version(name, details.version.to_string()),
        dir.to_str().unwrap()
    ));
    let output = command.output()?;
    spinner.finish_and_clear();

    if !output.status.success() {
        debug!(
            "Command failed, stderr is:\n{}",
            String::from_utf8_lossy(&output.stderr).to_string()
        );
        debug!("Exit code is {:?}", output.status.code());
        // TODO: Make this be a correct error
        throw!(ErrorDetails::NpmViewMetadataFetchError {
            package: name.to_string(),
        });
    }

    debug!("Downloading {} from {}, to {}", &spec, &url, path.display());
    Tarball::fetch(url, path).with_context(download_tool_error(spec, url.to_string()))
}

// build a command to run `npm pack` with json output
fn npm_pack_command_for(
    name: &str,
    version: &str,
    session: &mut Session,
    current_dir: Option<&Path>,
) -> Fallible<ToolCommand> {
    let args = vec![
        OsString::from("pack"),
        OsString::from("--json"),
        OsString::from(format!("{}@{}", name, version)),
    ];
    run::npm::command(args, session, current_dir)
}

fn unpack_archive(archive: Box<Archive>, name: &str, version: &Version) -> Fallible<()> {
    let temp = create_staging_dir()?;
    debug!("Unpacking {} into '{}'", name, temp.path().display());

    let bar = progress_bar(
        archive.origin(),
        &tool_version(&name, &version),
        archive
            .uncompressed_size()
            .unwrap_or(archive.compressed_size()),
    );

    archive
        .unpack(temp.path(), &mut |_, read| {
            bar.inc(read as u64);
        })
        .with_context(|_| ErrorDetails::UnpackArchiveError {
            tool: name.into(),
            version: version.to_string(),
        })?;

    let image_dir = path::package_image_dir(&name, &version.to_string())?;
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

    bar.finish_and_clear();

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
            return Ok(entry.path().to_path_buf());
        }
    }
    // there is more than just a single directory here, something is wrong
    Err(ErrorDetails::PackageUnpackError.into())
}
