//! Provides types for installing packages to the user toolchain.

use std::fs::rename;
use std::process::{Command, ExitStatus, Stdio};
use std::ffi::OsStr;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::io;
use std::str;
use std::fs::read_dir;
use std::path::Path;
use std::path::PathBuf;

use readext::ReadExt;
use sha1::{Sha1, Digest};
use hex;
use semver::Version;

use crate::inventory::RegistryFetchError;
use crate::style::progress_spinner;
use crate::version::VersionSpec;
use crate::distro::Fetched;
use crate::path;
use archive::Tarball;
use crate::distro::error::DownloadError;
use crate::tool::ToolSpec;
use crate::style::{progress_bar, Action};
use tempfile::tempdir_in;
use crate::fs::ensure_containing_dir_exists;
use crate::platform::PlatformSpec;
use crate::manifest::Manifest;
use crate::session::Session;
use crate::project::DepPackageReadError;
use crate::shim;
use crate::platform::Image;
use crate::fs::read_file_opt;
use archive::Archive;

use notion_fail::{throw, ExitCode, Fallible, NotionFail, ResultExt};
use failure::Fail;
use notion_fail_derive::*;

pub(crate) mod serial;

#[cfg(feature = "mock-network")]
use mockito;
use serde_json;

cfg_if::cfg_if! {
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

/// Thrown when a user tries to install or fetch a package with no executables.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Package has no binaries or executables - nothing to do")]
#[notion_fail(code = "InvalidArguments")]
pub struct PackageHasNoExecutablesError;

/// Thrown when a package has been unpacked but is not formed correctly.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Package unpack error: Could not determine unpack directory name")]
#[notion_fail(code = "ConfigurationError")]
pub struct PackageUnpackError;

/// Thrown when package install command fails to execute.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Error executing package install command: {}", error)]
#[notion_fail(code = "FileSystemError")]
pub (crate) struct PackageInstallIoError {
    error: String,
}

impl PackageInstallIoError {
    pub(crate) fn from_io_error(error: &io::Error) -> Self {
        if let Some(inner_err) = error.get_ref() {
            PackageInstallIoError {
                error: inner_err.to_string(),
            }
        } else {
            PackageInstallIoError {
                error: error.to_string(),
            }
        }
    }
}

/// Thrown when package install command is not successful.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Command `{:?}` failed with status {}", cmd, status)]
#[notion_fail(code = "FileSystemError")]
pub (crate) struct PackageInstallFailedError {
    cmd: String,
    status: ExitStatus,
}

impl PackageInstallFailedError {
    pub(crate) fn new(cmd: String, status: ExitStatus) -> Self {
        PackageInstallFailedError {
            cmd,
            status,
        }
    }
}

/// Thrown when package tries to install a binary that is already installed.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Conflict with bin '{}' already installed by '{}' version {}", bin_name, package, version)]
#[notion_fail(code = "FileSystemError")]
pub (crate) struct BinaryAlreadyInstalledError {
    bin_name: String,
    package: String,
    version: String,
}

impl BinaryAlreadyInstalledError {
    pub(crate) fn new(bin_name: String, package: String, version: Version) -> Self {
        BinaryAlreadyInstalledError {
            bin_name,
            package,
            version: version.to_string(),
        }
    }
}

/// A provisioned Package distribution.
//#[derive(Debug)]
pub struct PackageDistro {
    name: String,
    shasum: String,
    tarball_url: String,
    version: Version,
    image_dir: PathBuf,
    shasum_file: PathBuf,
    distro_file: PathBuf,
}

/// A package version.
// #[derive(Eq, PartialEq, Clone, Debug)]
pub struct PackageVersion {
    pub name: String,
    pub version: Version,
    // map of binary names to locations
    pub bins: HashMap<String, String>,
    image_dir: PathBuf,
}

/// Programs used to install packages.
enum Installer {
    Npm,
    Yarn,
}

/// Configuration information about an installed package.
pub struct PackageConfig {
    /// The package name
    name: String,
    /// The package version
    version: Version,
    /// The platform used to install this package
    platform: PlatformSpec,
    /// The binaries installed by this package
    bins: Vec<String>,
}

/// Configuration information about an installed binary from a package.
pub struct BinConfig {
    /// The binary name
    name: String,
    /// The package that installed this binary
    package: String,
    /// The package version
    version: Version,
    /// The relative path of the binary in the installed package
    path: String,
    /// The platform used to install this binary
    platform: PlatformSpec,
}

impl PackageDistro {
    pub fn new(name: String, shasum: String, version: Version, tarball_url: String) -> Fallible<Self> {
        Ok(PackageDistro {
            name: name.clone(),
            shasum,
            version: version.clone(),
            tarball_url,
            image_dir: path::package_image_dir(&name, &version.to_string())?,
            distro_file: path::package_distro_file(&name, &version.to_string())?,
            shasum_file: path::package_distro_shasum(&name, &version.to_string())?,
        })
    }

    pub fn fetch(&self) -> Fallible<Fetched<PackageVersion>> {
        let archive = self.load_or_fetch_archive()?;

        let bar = progress_bar(
            Action::Fetching,
            &format!("{}-v{}", self.name, self.version),
            archive.uncompressed_size().unwrap_or(archive.compressed_size()),
        );

        let temp = tempdir_in(path::tmp_dir()?).unknown()?;
        archive
            .unpack(temp.path(), &mut |_, read| {
                bar.inc(read as u64);
            })
            .unknown()?;
        bar.finish();

        ensure_containing_dir_exists(&self.image_dir)?;

        let unpack_dir = find_unpack_dir(temp.path())?;
        rename(unpack_dir, &self.image_dir).unknown()?;

        // save the shasum in a file
        let mut f = File::create(&self.shasum_file).unknown()?;
        f.write_all(self.shasum.as_bytes()).unknown()?;
        f.sync_all().unknown()?;

        let pkg_info = Manifest::for_dir(&self.image_dir).with_context(DepPackageReadError::from_error)?;
        let bin_map = pkg_info.bin;
        if bin_map.is_empty() {
            throw!(PackageHasNoExecutablesError);
        }

        for (bin_name, bin_path) in bin_map.iter() {
            // check for conflicts with installed bins
            // some packages may install bins with the same name
            let bin_config_file = path::user_tool_bin_config(&bin_name)?;
            if bin_config_file.exists() {
                let bin_config = BinConfig::from_file(bin_config_file)?;
                throw!(BinaryAlreadyInstalledError::new(bin_name.to_string(), bin_config.package, bin_config.version));
            }
        }

        Ok(Fetched::Now(PackageVersion::new(
            self.name.clone(),
            self.version.clone(),
            bin_map,
        )?))
    }

    /// Loads the package tarball from disk, or fetches from URL.
    fn load_or_fetch_archive(&self) -> Fallible<Box<Archive>> {
        // try to use existing downloaded package
        if self.downloaded_pkg_is_ok()? {
            println!("downloaded package is OK, using that");
            Tarball::load(File::open(&self.distro_file).unknown()?).unknown()
        } else {
            println!("downloaded package is NOT OK, fetching");
            // otherwise have to download
            ensure_containing_dir_exists(&self.distro_file)?;
            Tarball::fetch(&self.tarball_url, &self.distro_file).with_context(
                DownloadError::for_tool(
                    ToolSpec::Package(
                        self.name.to_string(),
                        VersionSpec::exact(&self.version)
                    ),
                    self.tarball_url.to_string()
                )
            )
        }
    }

    /// Verify downloaded package, returning a PackageVersion if it is ok.
    fn downloaded_pkg_is_ok(&self) -> Fallible<bool> {
        let mut buffer = Vec::new();

        if let Ok(mut distro) = File::open(&self.distro_file) {

            if let Some(stored_shasum) = read_file_opt(&self.shasum_file).unknown()? {
                println!("read shasum from disk: {}", stored_shasum);

                distro.read_to_end(&mut buffer).unknown()?;
                println!("read distro file");

                // calculate the shasum
                let mut hasher = Sha1::new();
                hasher.input(buffer);
                let result = hasher.result();
                println!("hashed that file");
                let calculated_shasum = hex::encode(&result);
                println!("calculated shasum: {}", calculated_shasum);

                return Ok(stored_shasum == calculated_shasum);
            }
        }

        println!("package is not valid, going to download");
        // something went wrong, package is not valid
        Ok(false)
    }

}

// Figure out the unpacked package directory name dynamically, because
// packages typically extract to a "package" directory, but not always
fn find_unpack_dir(in_dir: &Path) -> Fallible<PathBuf> {
    let mut dirs = Vec::new();
    for entry in read_dir(in_dir).unknown()? {
        let entry = entry.unknown()?;
        dirs.push(entry.path());
    }
    if dirs.len() == 1 {
        Ok(dirs[0].to_path_buf())
    } else {
        // there is more than just a directory here, something is wrong
        throw!(PackageUnpackError);
    }
}

impl PackageVersion {
    pub fn new(name: String, version: Version, bins: HashMap<String, String>) -> Fallible<Self>{
        Ok(PackageVersion {
            name: name.clone(),
            version: version.clone(),
            bins,
            image_dir: path::package_image_dir(&name, &version.to_string())?,
        })
    }

    // parse the "engines" string to a VersionSpec, for matching against available Node versions
    pub fn engines_spec(&self) -> Fallible<VersionSpec> {
        let manifest = Manifest::for_dir(&self.image_dir)?;
        let engines = match manifest.engines() {
            Some(e) => e,
            None=> "*".to_string(), // if nothing specified, match all versions of Node
        };
        Ok(VersionSpec::Semver(VersionSpec::parse_requirements(engines)?))
    }

    pub fn install(&self, platform: &PlatformSpec, session: &mut Session) -> Fallible<()> {
        let image = platform.checkout(session)?;
        // use yarn if it is installed, otherwise default to npm
        let mut install_cmd = if let Some(ref _yarn) = image.yarn {
            install_command_for(Installer::Yarn, &self.image_dir.clone().into_os_string(), &image.path()?)
        } else {
            install_command_for(Installer::Npm, &self.image_dir.clone().into_os_string(), &image.path()?)
        };

        let output = install_cmd.output().with_context(PackageInstallIoError::from_io_error)?;
        if !output.status.success() {
            throw!(PackageInstallFailedError::new(format!("{:?}", install_cmd), output.status));
        }

        self.write_config_and_shims(&platform)?;

        Ok(())
    }

    fn package_config(&self, platform_spec: &PlatformSpec) -> PackageConfig {
        PackageConfig {
            name: self.name.to_string(),
            version: self.version.clone(),
            platform: platform_spec.clone(),
            bins: self.bins.iter().map(|(name, _path)| name.to_string()).collect(),
        }
    }

    fn bin_config(&self, bin_name: String, bin_path: String, platform_spec: &PlatformSpec) -> BinConfig {
        BinConfig {
            name: bin_name,
            package: self.name.to_string(),
            version: self.version.clone(),
            path: bin_path,
            platform: platform_spec.clone(),
        }
    }

    fn write_config_and_shims(&self, platform_spec: &PlatformSpec) -> Fallible<()> {
        self.package_config(&platform_spec).to_serial().write()?;
        for (bin_name, bin_path) in self.bins.iter() {
            self.bin_config(bin_name.to_string(), bin_path.to_string(), &platform_spec).to_serial().write()?;
            // write the link to launchscript/bin
            shim::create(&bin_name)?;
        }
        Ok(())
    }
}

impl Installer {
    pub fn cmd(&self) -> Command {
        match self {
            Installer::Npm => {
                let mut command = Command::new("npm");
                command.args(&["install", "--only=production"]);
                command
            }
            Installer::Yarn => {
                let mut command = Command::new("yarn");
                command.args(&["install", "--production"]);
                command
            }
        }
    }
}

/// Information about a user tool.
pub struct UserTool {
    pub bin_path: PathBuf,
    pub image: Image,
}

impl UserTool {
    pub fn from_config(bin_config: BinConfig, session: &mut Session) -> Fallible<Option<Self>> {
        let image_dir = path::package_image_dir(&bin_config.package, &bin_config.version.to_string())?;
        // canonicalize because path is relative, and sometimes uses '.' char
        let bin_path = image_dir.join(bin_config.path).canonicalize().unknown()?;

        Ok(Some(UserTool {
            bin_path,
            image: bin_config.platform.checkout(session)?,
        }))
    }
}

pub fn user_tool(tool_name: &str, session: &mut Session) -> Fallible<Option<UserTool>> {
    let bin_config_file = path::user_tool_bin_config(tool_name)?;
    if bin_config_file.exists() {
        let bin_config = BinConfig::from_file(bin_config_file)?;
        Ok(UserTool::from_config(bin_config, session)?)
    } else {
        Ok(None) // no config means the tool is not installed
    }
}


// build a package install command using the specified directory and path
fn install_command_for(installer: Installer, in_dir: &OsStr, path_var: &OsStr) -> Command {
    let mut command = installer.cmd();
    command.current_dir(in_dir);
    command.env("PATH", path_var);
    // connect stdout and stderr to the current stdout and stderr for this process
    // (so the user can see the install progress in real time)
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());
    command
}

/// Information about a package.
pub struct NpmPackage;

/// Index of versions of a specific package.
pub struct PackageIndex {
    latest: Version,
    entries: Vec<PackageEntry>,
}

#[derive(Debug)]
pub struct PackageEntry {
    version: Version,
    tarball: String,
    shasum: String,
}

impl NpmPackage {

    // TODO: should this be method of PackageDistro?
    // (so I don't have to pass around name, would be nice...)
    pub fn resolve_public(name: &String, matching: &VersionSpec) -> Fallible<PackageDistro> {
        let index: PackageIndex = resolve_package_metadata(name)?.into_index()?;

        let matching_package_entry = index.match_something(matching);
        if let Some(entry) = matching_package_entry {
            Ok(PackageDistro::new(
                name.to_string(),
                entry.shasum,
                entry.version,
                entry.tarball
            )?)
        } else {
            throw!(NoPackageFoundError {
                name: name.to_string(),
                matching: matching.clone()
            })
        }
    }
}

impl PackageIndex {
    /// Try to find a match for the input VersionSpec in this index.
    pub fn match_something(self, matching: &VersionSpec) -> Option<PackageEntry> {
        match *matching {
            VersionSpec::Latest => {
                let latest = self.latest.clone();
                self.match_package_version(|&PackageEntry { version: ref v, .. }| &latest == v)
            }
            VersionSpec::Semver(ref matching) => {
                self.match_package_version(|&PackageEntry { version: ref v, .. }| matching.matches(v))
            }
            VersionSpec::Exact(ref exact) => {
                self.match_package_version(|&PackageEntry { version: ref v, .. }| exact == v)
            }
        }
    }

    // use the input predicate to match a package in the index
    fn match_package_version(self, predicate: impl Fn(&PackageEntry) -> bool) -> Option<PackageEntry> {
        let mut entries = self.entries.into_iter();
        entries.find(predicate)
    }
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

    let metadata: serial::PackageMetadata = serde_json::de::from_str(&response_text).unknown()?;

    spinner.finish_and_clear();
    Ok(metadata)
}

