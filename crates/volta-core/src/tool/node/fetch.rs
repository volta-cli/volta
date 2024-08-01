//! Provides fetcher for Node distributions

use std::fs::{read_to_string, write, File};
use std::path::{Path, PathBuf};

use super::NodeVersion;
use crate::error::{Context, ErrorKind, Fallible};
use crate::fs::{create_staging_dir, create_staging_file, rename};
use crate::hook::ToolHooks;
use crate::layout::volta_home;
use crate::style::{progress_bar, tool_version};
use crate::tool::{self, download_tool_error, Node};
use crate::version::{parse_version, VersionSpec};
use archive::{self, Archive};
use cfg_if::cfg_if;
use fs_utils::ensure_containing_dir_exists;
use log::debug;
use node_semver::Version;
use serde::Deserialize;

cfg_if! {
    if #[cfg(feature = "mock-network")] {
        // TODO: We need to reconsider our mocking strategy in light of mockito deprecating the
        // SERVER_URL constant: Since our acceptance tests run the binary in a separate process,
        // we can't use `mockito::server_url()`, which relies on shared memory.
        fn public_node_server_root() -> String {
            #[allow(deprecated)]
            mockito::SERVER_URL.to_string()
        }
    } else {
        fn public_node_server_root() -> String {
            "https://nodejs.org/dist".to_string()
        }
    }
}

fn npm_manifest_path(version: &Version) -> PathBuf {
    let mut manifest = PathBuf::from(Node::archive_basename(version));

    #[cfg(unix)]
    manifest.push("lib");

    manifest.push("node_modules");
    manifest.push("npm");
    manifest.push("package.json");

    manifest
}

pub fn fetch(version: &Version, hooks: Option<&ToolHooks<Node>>) -> Fallible<NodeVersion> {
    let home = volta_home()?;
    let node_dir = home.node_inventory_dir();
    let cache_file = node_dir.join(Node::archive_filename(version));

    let (archive, staging) = match load_cached_distro(&cache_file) {
        Some(archive) => {
            debug!(
                "Loading {} from cached archive at '{}'",
                tool_version("node", version),
                cache_file.display()
            );
            (archive, None)
        }
        None => {
            let staging = create_staging_file()?;
            let remote_url = determine_remote_url(version, hooks)?;
            let archive = fetch_remote_distro(version, &remote_url, staging.path())?;
            (archive, Some(staging))
        }
    };

    let node_version = unpack_archive(archive, version)?;

    if let Some(staging_file) = staging {
        ensure_containing_dir_exists(&cache_file).with_context(|| {
            ErrorKind::ContainingDirError {
                path: cache_file.clone(),
            }
        })?;
        staging_file
            .persist(cache_file)
            .with_context(|| ErrorKind::PersistInventoryError {
                tool: "Node".into(),
            })?;
    }

    Ok(node_version)
}

/// Unpack the node archive into the image directory so that it is ready for use
fn unpack_archive(archive: Box<dyn Archive>, version: &Version) -> Fallible<NodeVersion> {
    let temp = create_staging_dir()?;
    debug!("Unpacking node into '{}'", temp.path().display());

    let progress = progress_bar(
        archive.origin(),
        &tool_version("node", version),
        archive.compressed_size(),
    );
    let version_string = version.to_string();

    archive
        .unpack(temp.path(), &mut |_, read| {
            progress.inc(read as u64);
        })
        .with_context(|| ErrorKind::UnpackArchiveError {
            tool: "Node".into(),
            version: version_string.clone(),
        })?;

    // Save the npm version number in the npm version file for this distro
    let npm_package_json = temp.path().join(npm_manifest_path(version));
    let npm = Manifest::version(&npm_package_json)?;
    save_default_npm_version(version, &npm)?;

    let dest = volta_home()?.node_image_dir(&version_string);
    ensure_containing_dir_exists(&dest)
        .with_context(|| ErrorKind::ContainingDirError { path: dest.clone() })?;

    rename(temp.path().join(Node::archive_basename(version)), &dest).with_context(|| {
        ErrorKind::SetupToolImageError {
            tool: "Node".into(),
            version: version_string,
            dir: dest.clone(),
        }
    })?;

    progress.finish_and_clear();

    // Note: We write these after the progress bar is finished to avoid display bugs with re-renders of the progress
    debug!("Saving bundled npm version ({})", npm);
    debug!("Installing node in '{}'", dest.display());

    Ok(NodeVersion {
        runtime: version.clone(),
        npm,
    })
}

/// Return the archive if it is valid. It may have been corrupted or interrupted in the middle of
/// downloading.
// ISSUE(#134) - verify checksum
fn load_cached_distro(file: &Path) -> Option<Box<dyn Archive>> {
    if file.is_file() {
        let file = File::open(file).ok()?;
        archive::load_native(file).ok()
    } else {
        None
    }
}

/// Determine the remote URL to download from, using the hooks if available
fn determine_remote_url(version: &Version, hooks: Option<&ToolHooks<Node>>) -> Fallible<String> {
    let distro_file_name = Node::archive_filename(version);
    match hooks {
        Some(&ToolHooks {
            distro: Some(ref hook),
            ..
        }) => {
            debug!("Using node.distro hook to determine download URL");
            hook.resolve(version, &distro_file_name)
        }
        _ => Ok(format!(
            "{}/v{}/{}",
            public_node_server_root(),
            version,
            distro_file_name
        )),
    }
}

/// Fetch the distro archive from the internet
fn fetch_remote_distro(
    version: &Version,
    url: &str,
    staging_path: &Path,
) -> Fallible<Box<dyn Archive>> {
    debug!("Downloading {} from {}", tool_version("node", version), url);
    archive::fetch_native(url, staging_path).with_context(download_tool_error(
        tool::Spec::Node(VersionSpec::Exact(version.clone())),
        url,
    ))
}

/// The portion of npm's `package.json` file that we care about
#[derive(Deserialize)]
struct Manifest {
    version: String,
}

impl Manifest {
    /// Parse the version out of a package.json file
    fn version(path: &Path) -> Fallible<Version> {
        let file = File::open(path).with_context(|| ErrorKind::ReadNpmManifestError)?;
        let manifest: Manifest =
            serde_json::de::from_reader(file).with_context(|| ErrorKind::ParseNpmManifestError)?;
        parse_version(manifest.version)
    }
}

/// Load the local npm version file to determine the default npm version for a given version of Node
pub fn load_default_npm_version(node: &Version) -> Fallible<Version> {
    let npm_version_file_path = volta_home()?.node_npm_version_file(&node.to_string());
    let npm_version =
        read_to_string(&npm_version_file_path).with_context(|| ErrorKind::ReadDefaultNpmError {
            file: npm_version_file_path,
        })?;
    parse_version(npm_version)
}

/// Save the default npm version to the filesystem for a given version of Node
fn save_default_npm_version(node: &Version, npm: &Version) -> Fallible<()> {
    let npm_version_file_path = volta_home()?.node_npm_version_file(&node.to_string());
    write(&npm_version_file_path, npm.to_string().as_bytes()).with_context(|| {
        ErrorKind::WriteDefaultNpmError {
            file: npm_version_file_path,
        }
    })
}
