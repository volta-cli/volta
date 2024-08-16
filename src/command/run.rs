use std::collections::HashMap;
use std::ffi::OsString;

use crate::command::Command;
use crate::common::{Error, IntoResult};
use log::warn;
use volta_core::error::{report_error, ExitCode, Fallible};
use volta_core::platform::{CliPlatform, InheritOption};
use volta_core::run::execute_tool;
use volta_core::session::{ActivityKind, Session};
use volta_core::tool::{node, npm, pnpm, yarn};

#[derive(Debug, clap::Args)]
pub(crate) struct Run {
    /// Set the custom Node version
    #[arg(long, value_name = "version")]
    node: Option<String>,

    /// Set the custom npm version
    #[arg(long, value_name = "version", conflicts_with = "bundled_npm")]
    npm: Option<String>,

    /// Forces npm to be the version bundled with Node
    #[arg(long, conflicts_with = "npm")]
    bundled_npm: bool,

    /// Set the custon pnpm version
    #[arg(long, value_name = "version", conflicts_with = "no_pnpm")]
    pnpm: Option<String>,

    /// Disables pnpm
    #[arg(long, conflicts_with = "pnpm")]
    no_pnpm: bool,

    /// Set the custom Yarn version
    #[arg(long, value_name = "version", conflicts_with = "no_yarn")]
    yarn: Option<String>,

    /// Disables Yarn
    #[arg(long, conflicts_with = "yarn")]
    no_yarn: bool,

    /// Set an environment variable (can be used multiple times)
    #[arg(long = "env", value_name = "NAME=value", num_args = 1)]
    envs: Vec<String>,

    /// The command to run, along with any arguments
    #[arg(
        allow_hyphen_values = true,
        trailing_var_arg = true,
        value_name = "COMMAND",
        required = true
    )]
    command_and_args: Vec<OsString>,
}

impl Command for Run {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Run);

        let envs = self.parse_envs();
        let platform = self.parse_platform(session)?;

        // Safety: At least one value is required for `command_and_args`, so there must be at
        // least one value in the list. If no value is provided, Clap will show a "required
        // argument missing" message and this function won't be called.
        let command = &self.command_and_args[0];
        let args = &self.command_and_args[1..];

        match execute_tool(command, args, &envs, platform, session).into_result() {
            Ok(()) => {
                session.add_event_end(ActivityKind::Run, ExitCode::Success);
                Ok(ExitCode::Success)
            }
            Err(Error::Tool(code)) => {
                session.add_event_tool_end(ActivityKind::Run, code);
                Ok(ExitCode::ExecutionFailure)
            }
            Err(Error::Volta(err)) => {
                report_error(env!("CARGO_PKG_VERSION"), &err);
                session.add_event_error(ActivityKind::Run, &err);
                session.add_event_end(ActivityKind::Run, err.exit_code());
                Ok(err.exit_code())
            }
        }
    }
}

impl Run {
    /// Builds a CliPlatform from the provided cli options
    ///
    /// Will resolve a semver / tag version if necessary
    fn parse_platform(&self, session: &mut Session) -> Fallible<CliPlatform> {
        let node = self
            .node
            .as_ref()
            .map(|version| node::resolve(version.parse()?, session))
            .transpose()?;

        let npm = match (self.bundled_npm, &self.npm) {
            (true, _) => InheritOption::None,
            (false, None) => InheritOption::Inherit,
            (false, Some(version)) => match npm::resolve(version.parse()?, session)? {
                None => InheritOption::Inherit,
                Some(npm) => InheritOption::Some(npm),
            },
        };

        let pnpm = match (self.no_pnpm, &self.pnpm) {
            (true, _) => InheritOption::None,
            (false, None) => InheritOption::Inherit,
            (false, Some(version)) => {
                InheritOption::Some(pnpm::resolve(version.parse()?, session)?)
            }
        };

        let yarn = match (self.no_yarn, &self.yarn) {
            (true, _) => InheritOption::None,
            (false, None) => InheritOption::Inherit,
            (false, Some(version)) => {
                InheritOption::Some(yarn::resolve(version.parse()?, session)?)
            }
        };

        Ok(CliPlatform {
            node,
            npm,
            pnpm,
            yarn,
        })
    }

    /// Convert the environment variable settings passed to the command line into a map
    ///
    /// We ignore any setting that doesn't have a value associated with it
    /// We also ignore the PATH environment variable as that is set when running a command
    fn parse_envs(&self) -> HashMap<&str, &str> {
        self.envs.iter().filter_map(|entry| {
            let mut key_value = entry.splitn(2, '=');

            match (key_value.next(), key_value.next()) {
                (None, _) => None,
                (Some(_), None) => None,
                (Some(key), _) if key.eq_ignore_ascii_case("PATH") => {
                    warn!("Ignoring {} environment variable as it will be overwritten when executing the command", key);
                    None
                }
                (Some(key), Some(value)) => Some((key, value)),
            }
        }).collect()
    }
}
