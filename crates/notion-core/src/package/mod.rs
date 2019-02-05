//! Provides types for installing packages to the user toolchain.

use std::fs::rename;
use std::rc::Rc;
use std::process::{Command, Stdio};
use std::ffi::OsStr;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use readext::ReadExt;

use inventory::RegistryFetchError;
use semver::Version;
use style::progress_spinner;
use version::VersionSpec;
// use config::Config;
use distro::Fetched;
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
use project::DepPackageReadError;
use toolchain::serial::Platform;
use shim;
use std::path::PathBuf;
use platform::Image;

use notion_fail::{ExitCode, Fallible, NotionFail, ResultExt};
use notion_fail::FailExt;

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

/// A package version.
// #[derive(Eq, PartialEq, Clone, Debug)]
pub struct PackageVersion {
    pub name: String,
    pub version: Version,
    // map of binary names to locations
    pub bins: HashMap<String, String>,
}

impl PackageDistro {
    pub fn fetch(&self) -> Fallible<Fetched<PackageVersion>> {
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
        rename(temp.path().join("package"), &dest).unknown()?;

        // TODO: different error for this
        let pkg_info = Manifest::for_dir(&dest).with_context(DepPackageReadError::from_error)?;
        let bin_map = pkg_info.bin;
        if bin_map.is_empty() {
            unimplemented!("TODO: Need to throw an error for this - user tool has no binaries");
        }

        // TODO: check for conflicts with installed bins
        // some packages may have bins with the same name
        // warn and ask the user what to do? or just fail? probably fail for now

        Ok(Fetched::Now(PackageVersion {
            name: self.name.clone(),
            version: self.version.clone(),
            bins: bin_map,
        }))
    }

    // TODO: description
    pub fn platform(pkg_version: &PackageVersion) -> Fallible<Option<Rc<PlatformSpec>>> {
        let package_dir = path::package_image_dir(&pkg_version.name, &pkg_version.version.to_string())?;
        let manifest = Manifest::for_dir(&package_dir)?;
        Ok(manifest.platform())
    }

    pub fn install(pkg_version: &PackageVersion, platform: &PlatformSpec, session: &mut Session) -> Fallible<()> {
        // will eventually change to the installed package directory
        let package_dir = path::package_image_dir(&pkg_version.name, &pkg_version.version.to_string())?;

        // checkout the image to use
        let image = platform.checkout(session)?;

        if let Some(ref _yarn) = image.yarn {
            // use yarn to install
            println!("Running `yarn install` in dir {:?}", &package_dir);
            let output = install_command_for("yarn", &package_dir.into_os_string(), &image.path()?)
                .output()
                .expect("Failed to execute `yarn install`");
            if !output.status.success() {
                eprintln!("Whoops! `yarn install` failed with status {}", output.status);
                unimplemented!("TODO: error that `yarn install` failed");
            }
        } else {
            // otherwise use npm
            println!("Running `npm install` in dir {:?}", &package_dir);
            let output = install_command_for("npm", &package_dir.into_os_string(), &image.path()?)
                .output()
                .expect("Failed to execute `npm install`");
            if !output.status.success() {
                eprintln!("Whoops! `npm install` failed with status {}", output.status);
                unimplemented!("TODO: error that `npm install` failed");
            }
        }

        write_platform_and_shims(&pkg_version, &platform)?;

        Ok(())
    }
}

// TODO:
pub struct UserTool {
    pub bin_path: PathBuf,
    pub image: Image,
}

impl UserTool {
    pub fn from_config(name: &str, session: &mut Session, src: &str) -> Fallible<Option<Self>> {
        if let Some(platform_spec) = Platform::from_json(src.to_string())?.into_image()? {
            Ok(Some(UserTool {
                bin_path: path::user_tool_bin_link(&name)?,
                image: platform_spec.checkout(session)?,
            }))
        } else {
            Ok(None)
        }
    }
}

pub fn user_tool(tool_name: &str, session: &mut Session) -> Fallible<Option<UserTool>> {
    let config_path = path::user_package_config_file(&tool_name)?;
    if config_path.exists() {
        let config_data = File::open(config_path).unknown()?.read_into_string().unknown()?;
        Ok(UserTool::from_config(&tool_name, session, &config_data)?)
    } else {
        Ok(None) // no config means the tool is not installed
    }
}


fn write_platform_and_shims(pkg_version: &PackageVersion, platform_spec: &PlatformSpec) -> Fallible<()> {
    // the platform information for the installed executables
    let src = platform_spec.to_serial().to_json()?;

    for (bin_name, bin_path) in pkg_version.bins.iter() {

        // write config
        let config_file_path = path::user_package_config_file(bin_name)?;
        ensure_containing_dir_exists(&config_file_path)?;
        // TODO: handle errors here, or throw known errors
        let mut file = File::create(&config_file_path).unknown()?;
        file.write_all(src.as_bytes()).unknown()?;

        // write the symlink to the binary
        // TODO: this should be part of the config data?
        let shim_file = path::user_tool_bin_link(bin_name)?;
        // canonicalize because path is relative, and sometimes uses '.' char
        let binary_file = path::package_image_dir(&pkg_version.name, &pkg_version.version.to_string())?.join(bin_path).canonicalize().unknown()?;
        ensure_containing_dir_exists(&shim_file)?;
        println!("{:?} ~> {:?}", shim_file, binary_file);
        // TODO: handle errors for this, like notion-core/src/shim.rs
        path::create_file_symlink(binary_file, shim_file).unknown()?;

        // write the link to launchscript/bin
        shim::create(&bin_name)?;
    }

    Ok(())
}

// TODO: description
fn install_command_for(exe: &str, in_dir: &OsStr, path_var: &OsStr) -> Command {
    let mut command = Command::new(exe);
    command.arg("install");
    command.arg("--only=production"); // TODO: npm only, but doesn't work for yarn
    command.arg("--global-style"); // TODO: npm only, but doesn't work for yarn
    // command.arg("--production"); // TODO: yarn only, deprecated in npm
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

