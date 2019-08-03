use std::ffi::{OsStr, OsString};
use std::iter::once;
use std::path::PathBuf;

use super::ToolCommand;
use crate::error::ErrorDetails;
use crate::path;
use crate::platform::PlatformSpec;
use crate::platform::Source;
use crate::session::{ActivityKind, Session};
use crate::tool::bin_full_path;
use crate::tool::{BinConfig, BinLoader};

use log::debug;
use volta_fail::{throw, Fallible};

pub(super) fn command<A>(exe: OsString, args: A, session: &mut Session) -> Fallible<ToolCommand>
where
    A: IntoIterator<Item = OsString>,
{
    session.add_event_start(ActivityKind::Binary);

    // first try to use the project toolchain
    if let Some(project) = session.project()? {
        // check if the executable is a direct dependency
        if project.has_direct_bin(&exe)? {
            // use the full path to the file
            let mut path_to_bin = project.local_bin_dir();
            path_to_bin.push(&exe);

            if !path_to_bin.is_file() {
                throw!(ErrorDetails::ProjectLocalBinaryNotFound {
                    command: path_to_bin.to_string_lossy().to_string(),
                });
            }

            debug!(
                "Found {} in project at '{}'",
                exe.to_string_lossy(),
                path_to_bin.display()
            );
            let path_to_bin = path_to_bin.as_os_str();

            if let Some(platform) = session.current_platform()? {
                match platform.source() {
                    Source::Project | Source::ProjectNodeDefaultYarn => {
                        debug!("Using node@{} from project configuration", platform.node())
                    }
                    Source::Default => {
                        debug!("Using node@{} from default configuration", platform.node())
                    }
                };

                let image = platform.checkout(session)?;
                let path = image.path()?;
                return Ok(ToolCommand::project_local(&path_to_bin, args, &path));
            }

            // if there's no platform available, pass through to existing PATH.
            debug!("Could not find Volta configuration, delegating to system");
            return ToolCommand::passthrough(&path_to_bin, args, ErrorDetails::NoPlatform);
        }
    }

    // try to use the user toolchain
    if let Some(user_tool) = DefaultBinary::from_name(&exe, session)? {
        let image = user_tool.platform.checkout(session)?;
        debug!(
            "Found default {} in '{}'",
            exe.to_string_lossy(),
            user_tool.bin_path.display()
        );
        debug!(
            "Using node@{} from binary configuration",
            image.node.runtime
        );

        let path = image.path()?;
        let tool_path = user_tool.bin_path.into_os_string();
        let cmd = match user_tool.loader {
            Some(loader) => ToolCommand::direct(
                loader.command.as_ref(),
                loader
                    .args
                    .iter()
                    .map(|arg| OsString::from(arg))
                    .chain(once(tool_path))
                    .chain(args),
                &path,
            ),
            None => ToolCommand::direct(&tool_path, args, &path),
        };
        return Ok(cmd);
    }

    // at this point, there is no project or user toolchain
    // Pass through to the existing PATH
    debug!(
        "Could not find {}, delegating to system",
        exe.to_string_lossy()
    );
    ToolCommand::passthrough(
        &exe,
        args,
        ErrorDetails::BinaryNotFound {
            name: exe.to_string_lossy().to_string(),
        },
    )
}

/// Information about the location and execution context of user-default binaries
///
/// Fetched from the config files in the Volta directory, represents the binary that is executed
/// when the user is outside of a project that has the given bin as a dependency.
pub struct DefaultBinary {
    pub bin_path: PathBuf,
    pub platform: PlatformSpec,
    pub loader: Option<BinLoader>,
}

impl DefaultBinary {
    pub fn from_config(bin_config: BinConfig, session: &mut Session) -> Fallible<Self> {
        let bin_path = bin_full_path(
            &bin_config.package,
            &bin_config.version,
            &bin_config.name,
            &bin_config.path,
        )?;

        // If the user does not have yarn set in the platform for this binary, use the default
        // This is necessary because some tools (e.g. ember-cli with the `--yarn` option) invoke `yarn`
        let platform = match bin_config.platform.yarn {
            Some(_) => bin_config.platform,
            None => {
                let yarn = session
                    .user_platform()?
                    .and_then(|ref plat| plat.yarn.clone());
                PlatformSpec {
                    yarn,
                    ..bin_config.platform
                }
            }
        };

        Ok(DefaultBinary {
            bin_path,
            platform,
            loader: bin_config.loader,
        })
    }

    pub fn from_name(tool_name: &OsStr, session: &mut Session) -> Fallible<Option<Self>> {
        let bin_config_file = match tool_name.to_str() {
            Some(name) => path::user_tool_bin_config(name)?,
            None => return Ok(None),
        };

        if bin_config_file.exists() {
            let bin_config = BinConfig::from_file(bin_config_file)?;
            DefaultBinary::from_config(bin_config, session).map(Some)
        } else {
            Ok(None) // no config means the tool is not installed
        }
    }
}
