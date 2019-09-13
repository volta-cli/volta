use std::env;
use std::ffi::OsString;

use structopt::StructOpt;
use which::which_in;

use volta_core::error::ErrorDetails;
use volta_core::platform::System;
use volta_core::run::binary::DefaultBinary;
use volta_core::session::{ActivityKind, Session};
use volta_fail::{ExitCode, Fallible, ResultExt};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Which {
    /// The binary to find, e.g. `node` or `npm`
    binary: String,
}

impl Command for Which {
    // 1. Start by checking if the user has a tool installed in the project or
    //    as a user default. If so, we're done.
    // 2. Otherwise, use the platform image and/or the system environment to
    //    determine a lookup path to run `which` in.
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Which);

        let bin = OsString::from(self.binary.as_str());

        let user_tool = DefaultBinary::from_name(&bin, session)?;
        let project_bin_path =
            session
                .project()?
                .and_then(|project| match project.has_direct_bin(&bin) {
                    Ok(true) => Some(project.local_bin_dir()),
                    _ => None,
                });

        let tool_path = match (user_tool, project_bin_path) {
            (Some(_), Some(project_bin_dir)) => Some(project_bin_dir.join(&bin)),
            (Some(tool), _) => Some(tool.bin_path),
            _ => None,
        };

        if let Some(path) = tool_path {
            println!("{}", path.to_string_lossy());

            let exit_code = ExitCode::Success;
            session.add_event_end(ActivityKind::Which, exit_code);
            return Ok(exit_code);
        }

        // Treat any error with obtaining the current platform image as if the image doesn't exist
        // However, errors in obtaining the current working directory or the System path should
        // still be treated as errors.
        let path = match session
            .current_platform()
            .unwrap_or(None)
            .and_then(|platform| platform.checkout(session).ok())
            .and_then(|image| image.path().ok())
        {
            Some(path) => path,
            None => System::path()?,
        };

        let cwd = env::current_dir().with_context(|_| ErrorDetails::CurrentDirError)?;
        let exit_code = match which_in(&bin, Some(path), cwd) {
            Ok(result) => {
                println!("{}", result.to_string_lossy());
                ExitCode::Success
            }
            Err(_) => {
                // `which_in` Will return an Err if it can't find the binary in the path
                // In that case, we don't want to print anything out, but we want to return
                // Exit Code 1 (ExitCode::UnknownError)
                ExitCode::UnknownError
            }
        };

        session.add_event_end(ActivityKind::Which, exit_code);
        Ok(exit_code)
    }
}
