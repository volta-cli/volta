//! Provides installation steps for 3rd-party packages, fetching their dependencies,
//! writing config files, and creating shims

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

use super::super::Spec;
use super::bin_full_path;
use crate::command::create_command;
use crate::error::ErrorDetails;
use crate::layout::volta_home;
use crate::manifest::BinManifest;
use crate::platform::{BinaryPlatformSpec, Image, PlatformSpec};
use crate::session::Session;
use crate::shim;
use crate::style::{progress_spinner, tool_version};
use crate::version::{parse_requirements, VersionSpec, VersionTag};
use atty::Stream;
use cmdline_words_parser::StrExt;
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use semver::Version;
use volta_fail::{throw, Fallible, ResultExt};

lazy_static! {
    // Note: Regex adapted from @zkochan/cmd-shim package used by Yarn
    // https://github.com/pnpm/cmd-shim/blob/bac160cc554e5157e4c5f5e595af30740be3519a/index.js#L42
    static ref SHEBANG: Regex = Regex::new(r#"^#!\s*(?:/usr/bin/env)?\s*(?P<exe>[^ \t]+) ?(?P<args>.*)$"#)
        .expect("Regex is valid");
}

// TODO: (#526) this does not belong in the `install` module, since we now need
//       to expose it *outside* this module for the sake of listing data about
//       installed packages.
/// Configuration information about an installed package.
///
/// This information will be stored in ~/.volta/tools/user/packages/<package>.json.
///
/// For an example, this looks like:
///
/// {
///   "name": "cowsay",
///   "version": "1.4.0",
///   "platform": {
///     "node": {
///       "runtime": "11.10.1",
///       "npm": "6.7.0"
///     },
///     "yarn": null
///   },
///   "bins": [
///     "cowsay",
///     "cowthink"
///   ]
/// }
#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub struct PackageConfig {
    /// The package name
    pub name: String,
    /// The package version
    pub version: Version,
    /// The platform used to install this package
    pub platform: BinaryPlatformSpec,
    /// The binaries installed by this package
    pub bins: Vec<String>,
}

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
    pub platform: BinaryPlatformSpec,
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

pub fn install(
    name: &str,
    version: &Version,
    session: &mut Session,
) -> Fallible<HashMap<String, String>> {
    let package_dir = volta_home()?.package_image_dir(name, &version.to_string());
    let bin_map = read_bins(name, version)?;
    let display = tool_version(name, version);

    let engine = determine_engine(&package_dir, &display)?;
    let platform = BinaryPlatformSpec {
        node: Spec::Node(engine).resolve(session)?.into(),
        npm: None,
        yarn: None,
    };
    let image = platform.checkout(session)?;

    install_dependencies(&package_dir, image, &display)?;
    write_configs(name, version, &platform, &bin_map)?;

    Ok(bin_map)
}

fn determine_engine(package_dir: &Path, display: &str) -> Fallible<VersionSpec> {
    let manifest = BinManifest::for_dir(package_dir)?;
    // if nothing specified, use the latest version of Node
    match manifest.engine {
        Some(engine) => {
            debug!(
                "Found 'engines.node' specification for {}: {}",
                display, engine
            );
            let req = parse_requirements(engine)?;
            Ok(VersionSpec::Tag(VersionTag::LtsRequirement(req)))
        }
        None => {
            debug!("No 'engines.node' found for {}, using LTS", display);
            Ok(VersionSpec::Tag(VersionTag::Lts))
        }
    }
}

fn write_configs(
    name: &str,
    version: &Version,
    platform: &BinaryPlatformSpec,
    bins: &HashMap<String, String>,
) -> Fallible<()> {
    super::serial::RawPackageConfig::from(PackageConfig {
        name: name.to_string(),
        version: version.clone(),
        platform: platform.clone(),
        bins: bins.keys().map(String::clone).collect(),
    })
    .write()?;

    for (bin_name, bin_path) in bins.iter() {
        let full_path = bin_full_path(name, version, bin_name, bin_path)?;
        let loader = determine_script_loader(bin_name, &full_path)?;
        super::serial::RawBinConfig::from(BinConfig {
            name: bin_name.clone(),
            package: name.to_string(),
            version: version.clone(),
            path: bin_path.clone(),
            platform: platform.clone(),
            loader,
        })
        .write()?;

        // create a link to the shim executable
        shim::create(&bin_name)?;

        os::set_executable_permissions(&full_path).with_context(|_| {
            ErrorDetails::ExecutablePermissionsError {
                bin: bin_name.clone(),
            }
        })?;
    }

    Ok(())
}

fn install_dependencies(package_dir: &Path, image: Image, display: &str) -> Fallible<()> {
    let mut command = build_install_command(package_dir, &image.path()?);
    debug!("Installing dependencies with command: {:?}", command);

    let spinner = progress_spinner(&format!("Installing dependencies for {}", display));
    let output = command
        .output()
        .with_context(|_| ErrorDetails::PackageInstallFailed)?;
    spinner.finish_and_clear();

    debug!(
        "[install stderr]\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    debug!(
        "[install stdout]\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    if output.status.success() {
        Ok(())
    } else {
        Err(ErrorDetails::PackageInstallFailed.into())
    }
}

fn build_install_command(in_dir: &Path, path: &OsStr) -> Command {
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
    command.current_dir(in_dir);
    command.env("PATH", path);

    command
}

/// Read a fetched package and generate a map of all the bins it provides
fn read_bins(name: &str, version: &Version) -> Fallible<HashMap<String, String>> {
    let image_dir = volta_home()?.package_image_dir(&name, &version.to_string());
    let pkg_info = BinManifest::for_dir(&image_dir)?;
    let bin_map = pkg_info.bin;
    if bin_map.is_empty() {
        throw!(ErrorDetails::NoPackageExecutables);
    }

    for (bin_name, _bin_path) in bin_map.iter() {
        // check for conflicts with installed bins
        // some packages may install bins with the same name
        let bin_config_file = volta_home()?.default_tool_bin_config(&bin_name);
        if bin_config_file.exists() {
            let bin_config = BinConfig::from_file(bin_config_file)?;
            // if the bin was installed by the package that is currently being installed,
            // that's ok - otherwise it's an error
            if name != bin_config.package {
                throw!(ErrorDetails::BinaryAlreadyInstalled {
                    bin_name: bin_name.clone(),
                    existing_package: bin_config.package,
                    new_package: name.to_string(),
                });
            }
        }
    }

    Ok(bin_map)
}

/// Read the script for a shebang loader. If found, return it so it will be stored in the config
///
/// This is needed on Windows because Windows doesn't support shebang loaders for scripts
/// On Unix, we need to do this to remove any potential erroneous \r characters that may
/// have accidentally been published in the script.
fn determine_script_loader(bin_name: &str, full_path: &Path) -> Fallible<Option<BinLoader>> {
    let script =
        File::open(full_path).with_context(|_| ErrorDetails::DetermineBinaryLoaderError {
            bin: bin_name.into(),
        })?;
    if let Some(Ok(first_line)) = BufReader::new(script).lines().next() {
        if let Some(caps) = SHEBANG.captures(&first_line) {
            // Note: `caps["args"]` will never panic, since "args" is a non-optional part of the match
            // So if there is a Regex match, then it will necessarily include the "args" group.
            let args = caps["args"]
                .trim()
                .to_string()
                .parse_cmdline_words()
                .map(String::from)
                .collect();
            return Ok(Some(BinLoader {
                command: caps["exe"].into(),
                args,
            }));
        }
    }

    Ok(None)
}

mod os {
    use cfg_if::cfg_if;
    use std::io;
    use std::path::Path;

    cfg_if! {
        if #[cfg(windows)] {
            /// On Windows, this isn't a concern as there is no concept of 'executable' permissions
            pub fn set_executable_permissions(_bin: &Path) -> io::Result<()> {
                Ok(())
            }
        } else if #[cfg(unix)] {
            use std::fs;
            use std::os::unix::fs::PermissionsExt;

            /// Ensure that a given binary has 'executable' permissions on Unix, otherwise we won't be able to call it
            pub fn set_executable_permissions(bin: &Path) -> io::Result<()> {
                let mut permissions = fs::metadata(bin)?.permissions();
                let mode = permissions.mode();

                if mode & 0o111 != 0o111 {
                    permissions.set_mode(mode | 0o111);
                    fs::set_permissions(bin, permissions)
                } else {
                    Ok(())
                }
            }
        }
    }
}
