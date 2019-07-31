use std::collections::HashMap;
use std::fs::{rename, write, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use super::super::download_tool_error;
use crate::error::ErrorDetails;
use crate::fs::{
    create_staging_dir, ensure_containing_dir_exists, ensure_dir_does_not_exist, read_dir_eager,
    read_file_opt,
};
use crate::manifest::BinManifest;
use crate::path;
use crate::platform::PlatformSpec;
use crate::style::{progress_bar, tool_version};
use crate::tool::{self, PackageDetails};
use crate::version::VersionSpec;
use archive::{Archive, Tarball};
use log::debug;
use semver::Version;
use sha1::{Digest, Sha1};
use volta_fail::{throw, Fallible, ResultExt};

/// Configuration information about an installed binary from a package.
///
/// This information will be stored in ~/.volta/tools/user/bins/<bin-name>.json.
///
/// For an example, this looks like:
///
/// {
///   "name": "cowsay",
///   "package": "cowsay",
///   "version": "1.4.0",
///   "path": "./cli.js",
///   "platform": {
///     "node": {
///       "runtime": "11.10.1",
///       "npm": "6.7.0"
///     },
///     "yarn": null,
///     "loader": {
///       "exe": "node",
///       "args": []
///     }
///   }
/// }
pub struct BinConfig {
    /// The binary name
    pub name: String,
    /// The package that installed this binary
    pub package: String,
    /// The package version
    pub version: Version,
    /// The relative path of the binary in the installed package
    pub path: String,
    /// The platform used to install this binary
    pub platform: PlatformSpec,
    /// The loader information for the script, if any
    pub loader: Option<BinLoader>,
}

/// Information about the Shebang script loader (e.g. `#!/usr/bin/env node`)
///
/// Only important for Windows at the moment, as Windows does not natively understand script
/// loaders, so we need to provide that behavior when calling a script that uses one
pub struct BinLoader {
    /// The command used to run a script
    pub command: String,
    /// Any additional arguments specified for the loader
    pub args: Vec<String>,
}

pub fn fetch(name: &str, details: &PackageDetails) -> Fallible<()> {
    let version_string = details.version.to_string();
    let cache_file = path::package_distro_file(&name, &version_string)?;
    let shasum_file = path::package_distro_shasum(&name, &version_string)?;

    if let Some(archive) = load_cached_distro(&cache_file, &shasum_file) {
        debug!(
            "Loading {} from cached archive at '{}'",
            tool_version(&name, &version_string),
            cache_file.display(),
        );
        unpack_archive(archive, name, &details.version)
    } else {
        let archive = fetch_remote_distro(
            tool::Spec::Package(name.into(), VersionSpec::exact(&details.version)),
            &details.tarball_url,
            &cache_file,
        )?;

        unpack_archive(archive, &name, &details.version)?;

        // Save the shasum in a file
        write(&shasum_file, details.shasum.as_bytes()).with_context(|_| {
            ErrorDetails::WritePackageShasumError {
                package: name.into(),
                version: version_string,
                file: shasum_file,
            }
        })
    }
    // Check for cache, if available then use that (unpack_archive)
    // Otherwise, download from remote source into the cache file
    // (unpack_archive)
    // Once complete, write the shasum file
}

pub fn generate_bin_map(name: &str, version: &Version) -> Fallible<HashMap<String, String>> {
    let image_dir = path::package_image_dir(&name, &version.to_string())?;
    let pkg_info = BinManifest::for_dir(&image_dir)?;
    let bin_map = pkg_info.bin;
    if bin_map.is_empty() {
        throw!(ErrorDetails::NoPackageExecutables);
    }

    for (bin_name, _bin_path) in bin_map.iter() {
        // check for conflicts with installed bins
        // some packages may install bins with the same name
        let bin_config_file = path::user_tool_bin_config(&bin_name)?;
        if bin_config_file.exists() {
            let bin_config = BinConfig::from_file(bin_config_file)?;
            // if the bin was installed by the package that is currently being installed,
            // that's ok - otherwise it's an error
            if name != bin_config.package {
                throw!(ErrorDetails::BinaryAlreadyInstalled {
                    bin_name: bin_name.into(),
                    existing_package: bin_config.package,
                    new_package: name.into(),
                });
            }
        }
    }

    Ok(bin_map)
}

fn load_cached_distro(file: &Path, shasum_file: &Path) -> Option<Box<dyn Archive>> {
    let mut distro = File::open(file).ok()?;
    let stored_shasum = read_file_opt(shasum_file).ok()??; // `??`: Err(_) *or* Ok(None) -> None

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

fn fetch_remote_distro(spec: tool::Spec, url: &str, path: &Path) -> Fallible<Box<Archive>> {
    debug!("Downloading {} from {}", &spec, &url);
    Tarball::fetch(url, path).with_context(download_tool_error(spec, url.to_string()))
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
    ensure_containing_dir_exists(&image_dir)?;
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
