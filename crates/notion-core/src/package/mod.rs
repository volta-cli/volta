//! Provides types for installing packages to the user toolchain.

use std::fs::rename;
use std::rc::Rc;
use std::process::{Command, Stdio};
use std::ffi::OsStr;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::str;
use std::fs::read_dir;
use std::path::Path;

use readext::ReadExt;
use sha1::{Sha1, Digest};
use hex;

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
use tempfile::tempdir_in;
use fs::ensure_containing_dir_exists;
use platform::PlatformSpec;
use manifest::Manifest;
use session::Session;
use project::DepPackageReadError;
use toolchain::serial::Platform;
use shim;
use std::path::PathBuf;
use platform::Image;
use fs::read_file_opt;
use archive::Archive;

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
        // bar.finish_and_clear();
        bar.finish();

        ensure_containing_dir_exists(&self.image_dir)?;

        let unpack_dir = find_unpack_dir(temp.path())?;
        rename(unpack_dir, &self.image_dir).unknown()?;

        // save the shasum in a file
        let mut f = File::create(&self.shasum_file).unknown()?;
        f.write_all(self.shasum.as_bytes()).unknown()?;
        f.sync_all().unknown()?;

        // TODO: different error for this
        let pkg_info = Manifest::for_dir(&self.image_dir).with_context(DepPackageReadError::from_error)?;
        let bin_map = pkg_info.bin;
        if bin_map.is_empty() {
            unimplemented!("TODO: Need to throw an error for this - user tool has no binaries");
        }

        // TODO: check for conflicts with installed bins
        // some packages may have bins with the same name
        // warn and ask the user what to do? or just fail? probably fail for now

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
        unimplemented!("TODO: throw error that there's more than just a directory here");
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
    // TODO: description
    pub fn platform(&self) -> Fallible<Option<Rc<PlatformSpec>>> {
        let manifest = Manifest::for_dir(&self.image_dir)?;
        Ok(manifest.platform())
    }

    pub fn install(&self, platform: &PlatformSpec, session: &mut Session) -> Fallible<()> {
        // checkout the image to use
        let image = platform.checkout(session)?;

        if let Some(ref _yarn) = image.yarn {
            // use yarn to install
            println!("Running `yarn install` in dir {:?}", &self.image_dir);
            let output = install_command_for("yarn", &self.image_dir.clone().into_os_string(), &image.path()?)
                .output()
                .expect("Failed to execute `yarn install`");
            if !output.status.success() {
                eprintln!("Whoops! `yarn install` failed with status {}", output.status);
                unimplemented!("TODO: error that `yarn install` failed");
            }
        } else {
            // otherwise use npm
            println!("Running `npm install` in dir {:?}", &self.image_dir);
            let output = install_command_for("npm", &self.image_dir.clone().into_os_string(), &image.path()?)
                .output()
                .expect("Failed to execute `npm install`");
            if !output.status.success() {
                eprintln!("Whoops! `npm install` failed with status {}", output.status);
                unimplemented!("TODO: error that `npm install` failed");
            }
        }

        self.write_platform_and_shims(&platform)?;

        Ok(())
    }

    fn write_platform_and_shims(&self, platform_spec: &PlatformSpec) -> Fallible<()> {
        // the platform information for the installed executables
        let src = platform_spec.to_serial().to_json()?;

        for (bin_name, bin_path) in self.bins.iter() {

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
            let binary_file = self.image_dir.join(bin_path).canonicalize().unknown()?;
            ensure_containing_dir_exists(&shim_file)?;
            println!("{:?} ~> {:?}", shim_file, binary_file);
            // TODO: handle errors for this, like notion-core/src/shim.rs
            path::create_file_symlink(binary_file, shim_file).unknown()?;

            // write the link to launchscript/bin
            shim::create(&bin_name)?;
        }

        Ok(())
    }
}

// TODO: description
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


// TODO: description
fn install_command_for(exe: &str, in_dir: &OsStr, path_var: &OsStr) -> Command {
    let mut command = Command::new(exe);
    command.arg("install");
    command.arg("--only=production"); // TODO: npm only, but doesn't work for yarn
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

