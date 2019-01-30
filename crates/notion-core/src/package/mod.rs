//! Provides types for installing packages to the user toolchain.

use std::fs::rename;
use std::rc::Rc;
use std::process::{Command, Stdio};
use std::ffi::OsStr;

use inventory::RegistryFetchError;
use semver::Version;
use style::progress_spinner;
use version::VersionSpec;
// use config::Config;
use distro::Fetched;
use distro::DistroVersion;
use path;
use archive::Tarball;
use distro::error::DownloadError;
use tool::ToolSpec;
use style::{progress_bar, Action};
use tempfile::tempdir;
use fs::ensure_containing_dir_exists;
use platform::PlatformSpec;
use manifest::Manifest;
use session::Session;

use notion_fail::{ExitCode, Fallible, NotionFail, ResultExt};
// use notion_fail::FailExt;

pub(crate) mod serial;

#[cfg(feature = "mock-network")]
use mockito;
use serde_json;

cfg_if! {
    if #[cfg(feature = "mock-network")] {
        fn public_package_registry_root() -> String {
            mockito::SERVER_URL.to_string()
        }
    } else {
        fn public_package_registry_root() -> String {
            "https://registry.npmjs.org".to_string()
        }
    }
}

/// Thrown when there is no Node version matching a requested semver specifier.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "No version of '{}' found for {}", name, matching)]
#[notion_fail(code = "NoVersionMatch")]
struct NoPackageFoundError {
    name: String,
    matching: VersionSpec,
}

/// A provisioned Package distribution.
//#[derive(Debug)]
pub struct PackageDistro {
    name: String,
    tarball_url: String,
    version: Version,
}

impl PackageDistro {
    pub fn fetch(&self) -> Fallible<Fetched<DistroVersion>> {
        // TODO: check for existing downloaded archive (can check shasum to verify)
        // TODO: check collection for existing unpacked stuff
        // if collection.contains(&self.version) {
        //     let npm = load_default_npm_version(&self.version)?;
        //     return Ok(Fetched::Already(DistroVersion::Node(self.version, npm)));
        // }

        // get archive info
        let distro_file = path::package_distro_file(&self.name, &self.version.to_string())?;
        ensure_containing_dir_exists(&distro_file)?;

        let archive = Tarball::fetch(&self.tarball_url, &distro_file)
            .with_context(DownloadError::for_tool(ToolSpec::Package(self.name.to_string(), VersionSpec::exact(&self.version)), self.tarball_url.to_string()))?;

        let bar = progress_bar(
            Action::Fetching,
            &format!("{}-v{}", self.name, self.version),
            archive.uncompressed_size().unwrap_or(archive.compressed_size()),
        );

        let temp = tempdir().unknown()?;
        archive
            .unpack(temp.path(), &mut |_, read| {
                bar.inc(read as u64);
            })
            .unknown()?;
        // bar.finish_and_clear();
        bar.finish();

        let dest = path::package_image_dir(&self.name, &self.version.to_string())?;
        ensure_containing_dir_exists(&dest)?;

        // packages typically extract to a "package" directory, but not necessarily
        // TODO: have to figure out the directory name dynamically
        rename(temp.path().join("package"), dest).unknown()?;
        Ok(Fetched::Now(DistroVersion::Package(self.name.clone(), self.version.clone())))
    }

    // TODO: how should this work?
    pub fn platform(distro: &DistroVersion) -> Fallible<Option<Rc<PlatformSpec>>> {
        // TODO: that should be better typed to avoid this match
        // TODO: until then, can use `if let`...
        let (name, version) = match distro {
            DistroVersion::Package(n, v) => (n, v),
            DistroVersion::Node(_, _) => unimplemented!("this stuff needs better typing"),
            DistroVersion::Yarn(_) => unimplemented!("this stuff needs better typing"),
            DistroVersion::Npm(_) => unimplemented!("this stuff needs better typing"),
        };
        let package_dir = path::package_image_dir(&name, &version.to_string())?;
        let manifest = Manifest::for_dir(&package_dir)?;
        Ok(manifest.platform())
    }

    pub fn install(distro: &DistroVersion, platform: &PlatformSpec, session: &mut Session) -> Fallible<()> {
        // TODO: better typing to avoid this match
        // TODO: until then, can use `if let`...
        if let DistroVersion::Package(name, version) = distro {
            // will eventually change to the installed package directory
            let package_dir = path::package_image_dir(&name, &version.to_string())?;

            // checkout the image to use
            let image = platform.checkout(session)?;

            if let Some(ref _yarn) = platform.yarn {
                // use yarn to install
                println!("Running `yarn install` in dir {:?}", &package_dir);
                let output = install_command_for("yarn", &package_dir.into_os_string(), &image.path()?)
                    .output()
                    .expect("Failed to execute `yarn install`");
                // TODO: check success/failure
                println!("status: {}", output.status);
                println!("success? {}", output.status.success());
            } else if let Some(ref _npm) = platform.npm {
                println!("Running `npm install` in dir {:?}", &package_dir);
                // otherwise use npm
                let output = install_command_for("npm", &package_dir.into_os_string(), &image.path()?)
                    .output()
                    .expect("Failed to execute `npm install`");
                // TODO: check success/failure
                println!("status: {}", output.status);
                println!("success? {}", output.status.success());
            } else {
                // TODO: figure out the default npm version?
                // that should already be part of the platform spec tho...
                unimplemented!("probably shouldn't get here, with no npm version");
            }

        } else {
            unimplemented!("this stuff needs better typing");
        }

        // TODO: then setup shims to the binaries

        Ok(())
    }
}

// TODO: docs
fn install_command_for(exe: &str, in_dir: &OsStr, path_var: &OsStr) -> Command {
    let mut command = Command::new(exe);
    command.arg("install");
    command.current_dir(in_dir);
    command.env("PATH", path_var);
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());
    command
}


/// Information about a package.
pub struct NpmPackage;

// TODO: description
pub struct PackageIndex {
    latest: Version,
    entries: Vec<PackageEntry>,
}

#[derive(Debug)]
pub struct PackageEntry {
    version: Version,
    tarball: String,
}

impl NpmPackage {

    // TODO: should this be &self method?
    // (so I don't have to pass around name, would be nice...)
    pub fn resolve_public(name: &String, matching: &VersionSpec) -> Fallible<PackageDistro> {
        let index: PackageIndex = resolve_package_metadata(name)?.into_index()?;

        // TODO: this matching should be a self method of PackageIndex?
        let entry_opt = match *matching {
            VersionSpec::Latest => {
                let latest = index.latest.clone();
                match_package_version(index, |&PackageEntry { version: ref v, .. }| &latest == v)?
            }
            VersionSpec::Semver(ref matching) => {
                match_package_version(index, |&PackageEntry { version: ref v, .. }| matching.matches(v))?
            }
            VersionSpec::Exact(ref exact) => {
                match_package_version(index, |&PackageEntry { version: ref v, .. }| exact == v)?
            }
        };

        if let Some(index) = entry_opt {
            Ok(PackageDistro { name: name.to_string(), version: index.version, tarball_url: index.tarball })
        } else {
            throw!(NoPackageFoundError {
                name: name.to_string(),
                matching: matching.clone()
            })
        }
    }
}

// TODO: this should be a self method?
fn match_package_version(index: PackageIndex, predicate: impl Fn(&PackageEntry) -> bool) -> Fallible<Option<PackageEntry>> {
    let mut entries = index.entries.into_iter();
    Ok(entries.find(predicate))
}


// TODO: this has side effects, so needs acceptance & smoke tests
fn resolve_package_metadata(name: &String) -> Fallible<serial::PackageMetadata> {
    let package_info_uri = format!("{}/{}", public_package_registry_root(), name);
    let spinner = progress_spinner(&format!(
            "Fetching package metadata: {}",
            package_info_uri
            ));
    let mut response: reqwest::Response =
        reqwest::get(package_info_uri.as_str())
        .with_context(RegistryFetchError::from_error)?;
    let response_text: String = response.text().unknown()?;

    // TODO: caching for this? see inventory::resolve_node_versions() for an example of that

    let metadata: serial::PackageMetadata = serde_json::de::from_str(&response_text).unknown()?;

    spinner.finish_and_clear();
    Ok(metadata)
}

