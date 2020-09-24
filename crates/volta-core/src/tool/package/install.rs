//! Provides installation steps for 3rd-party packages, fetching their dependencies,
//! writing config files, and creating shims

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

use super::super::node;
use super::bin_full_path;
use super::metadata::{BinConfig, BinLoader, PackageConfig, RawBinConfig, RawPackageConfig};
use crate::command::create_command;
use crate::error::{Context, ErrorKind, Fallible};
use crate::fs::set_executable;
use crate::layout::volta_home;
use crate::manifest::BinManifest;
use crate::platform::{Image, PlatformSpec};
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

lazy_static! {
    // Note: Regex adapted from @zkochan/cmd-shim package used by Yarn
    // https://github.com/pnpm/cmd-shim/blob/bac160cc554e5157e4c5f5e595af30740be3519a/index.js#L42
    static ref SHEBANG: Regex = Regex::new(r#"^#!\s*(?:/usr/bin/env)?\s*(?P<exe>[^ \t]+) ?(?P<args>.*)$"#)
        .expect("Regex is valid");
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
    let platform = PlatformSpec {
        node: node::resolve(engine, session)?,
        npm: None,
        yarn: None,
    };
    let image = platform.as_binary().checkout(session)?;

    install_dependencies(&package_dir, image, &display)?;
    write_configs(name, version, &platform, &bin_map)?;

    Ok(bin_map)
}

fn determine_engine(package_dir: &Path, display: &str) -> Fallible<VersionSpec> {
    let manifest = BinManifest::for_dir(package_dir)?;
    // if nothing specified, use the LTS version of Node
    match manifest.engine {
        Some(engine) => {
            debug!(
                "Found 'engines.node' specification for {}: {}",
                display, engine
            );
            match parse_requirements(engine) {
                Ok(req) => Ok(VersionSpec::Tag(VersionTag::LtsRequirement(req))),
                Err(_) => {
                    debug!(
                        "Fail to parse 'engines.node' found for {}, using LTS instead",
                        display
                    );
                    Ok(VersionSpec::Tag(VersionTag::Lts))
                }
            }
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
    platform: &PlatformSpec,
    bins: &HashMap<String, String>,
) -> Fallible<()> {
    RawPackageConfig::from(PackageConfig {
        name: name.to_string(),
        version: version.clone(),
        platform: platform.clone(),
        bins: bins.keys().map(String::clone).collect(),
    })
    .write()?;

    for (bin_name, bin_path) in bins.iter() {
        let full_path = bin_full_path(name, version, bin_name, bin_path)?;
        let loader = determine_script_loader(bin_name, &full_path)?;
        RawBinConfig::from(BinConfig {
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

        set_executable(&full_path).with_context(|| ErrorKind::ExecutablePermissionsError {
            bin: bin_name.clone(),
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
        .with_context(|| ErrorKind::PackageDependenciesInstallFailed)?;
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
        Err(ErrorKind::PackageDependenciesInstallFailed.into())
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
        return Err(ErrorKind::NoPackageExecutables.into());
    }

    for (bin_name, _bin_path) in bin_map.iter() {
        // check for conflicts with installed bins
        // some packages may install bins with the same name
        let bin_config_file = volta_home()?.default_tool_bin_config(&bin_name);
        match BinConfig::from_file_if_exists(bin_config_file)? {
            None => (),
            Some(bin_config) => {
                // if the bin was installed by the package that is currently being installed,
                // that's ok - otherwise it's an error
                if name != bin_config.package {
                    return Err(ErrorKind::BinaryAlreadyInstalled {
                        bin_name: bin_name.clone(),
                        existing_package: bin_config.package,
                        new_package: name.to_string(),
                    }
                    .into());
                }
            }
        };
    }

    Ok(bin_map)
}

/// Read the script for a shebang loader. If found, return it so it will be stored in the config
///
/// This is needed on Windows because Windows doesn't support shebang loaders for scripts
/// On Unix, we need to do this to remove any potential erroneous \r characters that may
/// have accidentally been published in the script.
fn determine_script_loader(bin_name: &str, full_path: &Path) -> Fallible<Option<BinLoader>> {
    let script = File::open(full_path).with_context(|| ErrorKind::DetermineBinaryLoaderError {
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
