use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use super::executor::{ToolCommand, ToolKind};
use super::{debug_active_image, debug_no_platform};
use crate::error::{ErrorKind, Fallible};
use crate::layout::volta_home;
use crate::platform::{Platform, Sourced, System};
use crate::session::Session;
use crate::tool::package::BinConfig;
use log::debug;

/// Determine the correct command to run for a 3rd-party binary
///
/// Will detect if we should delegate to the project-local version or use the default version
pub(super) fn command(
    exe: &OsStr,
    args: &[OsString],
    session: &mut Session,
) -> Fallible<ToolCommand> {
    let bin = exe.to_string_lossy().to_string();
    // First try to use the project toolchain
    if let Some(project) = session.project()? {
        // Check if the executable is a direct dependency
        if project.has_direct_bin(exe)? {
            let path_to_bin =
                project
                    .find_bin(exe)
                    .ok_or_else(|| ErrorKind::ProjectLocalBinaryNotFound {
                        command: exe.to_string_lossy().to_string(),
                    })?;

            debug!("Found {} in project at '{}'", bin, path_to_bin.display());

            let platform = Platform::current(session)?;
            return Ok(ToolCommand::new(
                path_to_bin,
                args,
                platform,
                ToolKind::ProjectLocalBinary(bin),
            ));
        }
    }

    // Try to use the default toolchain
    if let Some(default_tool) = DefaultBinary::from_name(exe, session)? {
        debug!(
            "Found default {} in '{}'",
            bin,
            default_tool.bin_path.display()
        );

        return Ok(ToolCommand::new(
            default_tool.bin_path,
            args,
            Some(default_tool.platform),
            ToolKind::DefaultBinary(bin),
        ));
    }

    // At this point, the binary is not known to Volta, so we have no platform to use to execute it
    // This should be rare, as anything we have a shim for should have a config file to load
    Ok(ToolCommand::new(
        exe,
        args,
        None,
        ToolKind::DefaultBinary(bin),
    ))
}

/// Determine the execution context (PATH and failure error message) for a project-local binary
pub(super) fn local_execution_context(
    tool: String,
    platform: Option<Platform>,
    session: &mut Session,
) -> Fallible<(OsString, ErrorKind)> {
    match platform {
        Some(plat) => {
            let image = plat.checkout(session)?;
            let path = image.path()?;
            debug_active_image(&image);

            Ok((
                path,
                ErrorKind::ProjectLocalBinaryExecError { command: tool },
            ))
        }
        None => {
            let path = System::path()?;
            debug_no_platform();

            Ok((path, ErrorKind::NoPlatform))
        }
    }
}

/// Determine the execution context (PATH and failure error message) for a default binary
pub(super) fn default_execution_context(
    tool: String,
    platform: Option<Platform>,
    session: &mut Session,
) -> Fallible<(OsString, ErrorKind)> {
    match platform {
        Some(plat) => {
            let image = plat.checkout(session)?;
            let path = image.path()?;
            debug_active_image(&image);

            Ok((path, ErrorKind::BinaryExecError))
        }
        None => {
            let path = System::path()?;
            debug_no_platform();

            Ok((path, ErrorKind::BinaryNotFound { name: tool }))
        }
    }
}

/// Information about the location and execution context of default binaries
///
/// Fetched from the config files in the Volta directory, represents the binary that is executed
/// when the user is outside of a project that has the given bin as a dependency.
pub struct DefaultBinary {
    pub bin_path: PathBuf,
    pub platform: Platform,
}

impl DefaultBinary {
    pub fn from_config(bin_config: BinConfig, session: &mut Session) -> Fallible<Self> {
        let package_dir = volta_home()?.package_image_dir(&bin_config.package);
        let mut bin_path = bin_config.manager.binary_dir(package_dir);
        bin_path.push(&bin_config.name);

        // If the user does not have yarn set in the platform for this binary, use the default
        // This is necessary because some tools (e.g. ember-cli with the `--yarn` option) invoke `yarn`
        let yarn = match bin_config.platform.yarn {
            Some(yarn) => Some(yarn),
            None => session
                .default_platform()?
                .and_then(|ref plat| plat.yarn.clone()),
        };
        let platform = Platform {
            node: Sourced::with_binary(bin_config.platform.node),
            npm: bin_config.platform.npm.map(Sourced::with_binary),
            yarn: yarn.map(Sourced::with_binary),
        };

        Ok(DefaultBinary { bin_path, platform })
    }

    /// Load information about a default binary by name, if available
    ///
    /// A `None` response here means that the tool information couldn't be found. Either the tool
    /// name is not a valid UTF-8 string, or the tool config doesn't exist.
    pub fn from_name(tool_name: &OsStr, session: &mut Session) -> Fallible<Option<Self>> {
        let bin_config_file = match tool_name.to_str() {
            Some(name) => volta_home()?.default_tool_bin_config(name),
            None => return Ok(None),
        };

        match BinConfig::from_file_if_exists(bin_config_file)? {
            Some(config) => DefaultBinary::from_config(config, session).map(Some),
            None => Ok(None),
        }
    }
}
