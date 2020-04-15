use std::ffi::OsString;

use crate::command::Command;
use crate::common::{Error, IntoResult};
use log::debug;
use structopt::StructOpt;
use volta_core::error::report_error;
use volta_core::platform::{CliPlatform, InheritOption};
use volta_core::run::execute_tool;
use volta_core::session::{ActivityKind, Session};
use volta_core::tool::{node, yarn};
use volta_fail::{ExitCode, Fallible};

#[derive(Debug, StructOpt)]
pub(crate) struct Run {
    /// Set the custom Node version
    #[structopt(long = "node", value_name = "version")]
    node: Option<String>,

    /// Set the custom Yarn version
    #[structopt(long = "yarn", value_name = "version", conflicts_with = "no_yarn")]
    yarn: Option<String>,

    /// Disables Yarn
    #[structopt(long = "no-yarn", conflicts_with = "yarn")]
    no_yarn: bool,

    /// Set an environment variable (can be used multiple times)
    #[structopt(long = "env", value_name = "NAME=value", raw(number_of_values = "1"))]
    envs: Vec<String>,

    #[structopt(parse(from_os_str))]
    /// The command to run
    command: OsString,

    #[structopt(parse(from_os_str))]
    /// Arguments to pass to the command
    args: Vec<OsString>,
}

impl Command for Run {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Run);

        let envs = parse_envs(self.envs);
        let platform = parse_platform(self.node, self.yarn, self.no_yarn, session)?;

        match execute_tool(&self.command, self.args, envs, platform, session).into_result() {
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

/// Convert the environment variable settings passed to the command line into (Key, Value) pairs
///
/// We ignore any setting that doesn't have a value associated with it
/// We also ignore the PATH environment variable as that is set when running a command
fn parse_envs(cli_values: Vec<String>) -> impl IntoIterator<Item = (String, String)> {
    cli_values.into_iter().filter_map(|mut entry| {
        entry.find('=').and_then(|index| {
            // After `split_off`, entry will contain only the key
            let value = entry.split_off(index);

            if entry.eq_ignore_ascii_case("PATH") {
                debug!("Skipping PATH environment variable as it will be overwritten to execute the command");
                None
            } else {
                Some((entry, value))
            }
        })
    })
}

/// Builds a CliPlatform from the provided cli options
///
/// Will resolve a semver / tag version if necessary
fn parse_platform(
    node: Option<String>,
    yarn: Option<String>,
    no_yarn: bool,
    session: &mut Session,
) -> Fallible<CliPlatform> {
    let node = node
        .map(|version| node::resolve(version.parse()?, session))
        .transpose()?;

    let yarn = match (no_yarn, yarn) {
        (true, _) => InheritOption::None,
        (false, None) => InheritOption::Inherit,
        (false, Some(version)) => InheritOption::Some(yarn::resolve(version.parse()?, session)?),
    };

    Ok(CliPlatform {
        node,
        npm: InheritOption::Inherit,
        yarn,
    })
}
