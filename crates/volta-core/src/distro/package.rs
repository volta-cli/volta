//! Provides types for installing packages to the user toolchain.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, rename, write, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use atty::Stream;
use cfg_if::cfg_if;
use hex;
use log::{debug, info};
use semver::Version;
use sha1::{Digest, Sha1};
use tempfile::tempdir_in;

use crate::command::create_command;
use crate::distro::{download_tool_error, Distro, Fetched};
use crate::error::ErrorDetails;
use crate::fs::{
    delete_dir_error, dir_entry_match, ensure_containing_dir_exists, ensure_dir_does_not_exist,
    read_dir_eager, read_file_opt,
};
use crate::hook::ToolHooks;
use crate::inventory::PackageCollection;
use crate::manifest::BinManifest;
use crate::path;
use crate::platform::{Image, PlatformSpec};
use crate::session::Session;
use crate::shim;
use crate::style::{progress_bar, progress_spinner, tool_version};
use crate::tool;
use crate::version::VersionSpec;
use archive::{Archive, Tarball};

cfg_if! {
    if #[cfg(windows)] {
        use cmdline_words_parser::StrExt;
        use regex::Regex;
        use std::io::{BufRead, BufReader};
    } else if #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
    }
}

use volta_fail::{throw, Fallible, ResultExt};

/// Programs used to install packages.
enum Installer {
    Npm,
    Yarn,
}

impl PackageVersion {
    pub fn new(name: String, version: Version, bins: HashMap<String, String>) -> Fallible<Self> {
        let image_dir = path::package_image_dir(&name, &version.to_string())?;
        Ok(PackageVersion {
            name,
            version,
            bins,
            image_dir,
        })
    }

    // parse the "engines" string to a VersionSpec, for matching against available Node versions
    pub fn engines_spec(&self) -> Fallible<VersionSpec> {
        let manifest = BinManifest::for_dir(&self.image_dir)?;
        // if nothing specified, can use any version of Node
        let engine = match manifest.engine {
            Some(ref engine) => {
                debug!(
                    "Found 'engines.node' specification for {}: {}",
                    tool_version(&self.name, &self.version),
                    engine
                );
                engine.clone()
            }
            None => {
                debug!(
                    "No 'engines.node' found for {}, using latest",
                    tool_version(&self.name, &self.version)
                );
                String::from("*")
            }
        };
        let spec = VersionSpec::parse_requirements(engine)?;
        Ok(VersionSpec::Semver(spec))
    }

    pub fn install(&self, platform: &PlatformSpec, session: &mut Session) -> Fallible<()> {
        let image = platform.checkout(session)?;
        // use yarn if it is installed, otherwise default to npm
        let installer = if image.yarn.is_some() {
            Installer::Yarn
        } else {
            Installer::Npm
        };

        let mut command =
            install_command_for(installer, self.image_dir.as_os_str(), &image.path()?);
        self.log_installing_dependencies(&command);

        let spinner = progress_spinner(&format!(
            "Installing dependencies for {}",
            tool_version(&self.name, &self.version)
        ));
        let output = command
            .output()
            .with_context(|_| ErrorDetails::PackageInstallFailed)?;
        spinner.finish_and_clear();

        self.log_dependency_install_stderr(&output.stderr);
        self.log_dependency_install_stdout(&output.stdout);

        if !output.status.success() {
            throw!(ErrorDetails::PackageInstallFailed);
        }

        self.write_config_and_shims(&platform)?;

        Ok(())
    }

    fn package_config(&self, platform_spec: &PlatformSpec) -> PackageConfig {
        PackageConfig {
            name: self.name.to_string(),
            version: self.version.clone(),
            platform: platform_spec.clone(),
            bins: self
                .bins
                .iter()
                .map(|(name, _path)| name.to_string())
                .collect(),
        }
    }

    fn bin_config(
        &self,
        bin_name: String,
        bin_path: String,
        platform_spec: PlatformSpec,
        loader: Option<BinLoader>,
    ) -> BinConfig {
        BinConfig {
            name: bin_name,
            package: self.name.to_string(),
            version: self.version.clone(),
            path: bin_path,
            platform: platform_spec,
            loader,
        }
    }

    fn write_config_and_shims(&self, platform_spec: &PlatformSpec) -> Fallible<()> {
        self.package_config(&platform_spec).to_serial().write()?;
        for (bin_name, bin_path) in self.bins.iter() {
            let full_path = bin_full_path(&self.name, &self.version, bin_name, bin_path)?;
            let loader = determine_script_loader(bin_name, &full_path)?;
            self.bin_config(
                bin_name.to_string(),
                bin_path.to_string(),
                platform_spec.clone(),
                loader,
            )
            .to_serial()
            .write()?;
            // create a link to the shim executable
            shim::create(&bin_name)?;

            // On Unix, ensure the executable file has correct permissions
            #[cfg(unix)]
            set_executable_permissions(&full_path).with_context(|_| {
                ErrorDetails::ExecutablePermissionsError {
                    bin: bin_name.clone(),
                }
            })?;
        }
        Ok(())
    }

    fn log_installing_dependencies(&self, command: &Command) {
        debug!("Installing dependencies with command: {:?}", command);
    }

    fn log_dependency_install_stderr(&self, output: &Vec<u8>) {
        debug!("[install stderr]\n{}", String::from_utf8_lossy(output));
    }

    fn log_dependency_install_stdout(&self, output: &Vec<u8>) {
        debug!("[install stdout]\n{}", String::from_utf8_lossy(output));
    }
}

impl Installer {
    pub fn cmd(&self) -> Command {
        match self {
            Installer::Npm => {
                let mut command = create_command("npm");
                command.args(&[
                    "install",
                    "--only=production",
                    "--loglevel=warn",
                    "--no-update-notifier",
                    "--no-audit",
                ]);

                if atty::is(Stream::Stdout) {
                    // npm won't detect the existence of a TTY since we are piping the output
                    // force the output to be colorized for when we send it to the user
                    command.arg("--color=always");
                }

                command
            }
            Installer::Yarn => {
                let mut command = create_command("yarn");
                command.args(&["install", "--production", "--non-interactive"]);
                command
            }
        }
    }
}

/// Ensure that a given binary has 'executable' permissions on Unix, otherwise we won't be able to call it
/// On Windows, this isn't a concern as there is no concept of 'executable' permissions
#[cfg(unix)]
fn set_executable_permissions(bin: &Path) -> io::Result<()> {
    let mut permissions = fs::metadata(bin)?.permissions();
    let mode = permissions.mode();

    if mode & 0o111 != 0o111 {
        permissions.set_mode(mode | 0o111);
        fs::set_permissions(bin, permissions)
    } else {
        Ok(())
    }
}

/// On Unix, shebang loaders work correctly, so we don't need to bother storing loader information
#[cfg(unix)]
fn determine_script_loader(_bin_name: &str, _full_path: &Path) -> Fallible<Option<BinLoader>> {
    Ok(None)
}

/// On Windows, we need to read the executable and try to find a shebang loader
/// If it exists, we store the loader in the BinConfig so that the shim can execute it correctly
#[cfg(windows)]
fn determine_script_loader(bin_name: &str, full_path: &Path) -> Fallible<Option<BinLoader>> {
    let script =
        File::open(full_path).with_context(|_| ErrorDetails::DetermineBinaryLoaderError {
            bin: bin_name.to_string(),
        })?;
    if let Some(Ok(first_line)) = BufReader::new(script).lines().next() {
        // Note: Regex adapted from @zkochan/cmd-shim package used by Yarn
        // https://github.com/pnpm/cmd-shim/blob/bac160cc554e5157e4c5f5e595af30740be3519a/index.js#L42
        let re = Regex::new(r#"^#!\s*(?:/usr/bin/env)?\s*(?P<exe>[^ \t]+) ?(?P<args>.*)$"#)
            .expect("Regex is valid");
        if let Some(caps) = re.captures(&first_line) {
            let args = caps["args"]
                .to_string()
                .parse_cmdline_words()
                .map(|word| word.to_string())
                .collect();
            return Ok(Some(BinLoader {
                command: caps["exe"].to_string(),
                args,
            }));
        }
    }
    Ok(None)
}

/// Build a package install command using the specified directory and path
fn install_command_for(installer: Installer, in_dir: &OsStr, path_var: &OsStr) -> Command {
    let mut command = installer.cmd();
    command.current_dir(in_dir).env("PATH", path_var);
    command
}

#[derive(Debug)]
pub struct PackageEntry {
    pub version: Version,
    pub tarball: String,
    pub shasum: String,
}
