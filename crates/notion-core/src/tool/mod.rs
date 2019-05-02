//! Traits and types for executing command-line tools.

use std::env::{self, args_os, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Debug, Display, Formatter};
use std::path::Path;
use std::process::{Command, ExitStatus};

use lazy_static::lazy_static;
use regex::Regex;

use notion_fail::{Fallible, ResultExt};
use validate_npm_package_name::{validate, Validity};

use crate::command::create_command;
use crate::env::UNSAFE_GLOBAL;
use crate::error::ErrorDetails;
use crate::path;
use crate::platform::System;
use crate::session::Session;
use crate::version::VersionSpec;

mod binary;
mod node;
mod npm;
mod npx;
mod yarn;

lazy_static! {
    static ref TOOL_SPEC_PATTERN: Regex =
        Regex::new("^(?P<name>(?:@([^/]+?)[/])?([^/]+?))(@(?P<version>.+))?$")
            .expect("regex is valid");
}

/// Specification for a tool and its associated version.
///
/// Since [`Ord`] is implemented for `ToolSpec`, we can use `.sort` on any
/// `Vec<ToolSpec>`, and the order of the enum variants in the declaration
/// determines the sorting order, which lets us guarantee (for example) that
/// Node will always be prioritized over other tools in the toolchain when
/// dealing with multiple tools.
///
/// [`Ord`]: https://doc.rust-lang.org/1.34.0/core/cmp/trait.Ord.html
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ToolSpec {
    Node(VersionSpec),
    Npm(VersionSpec),
    Yarn(VersionSpec),
    Package(String, VersionSpec),
}

impl ToolSpec {
    pub fn from_str_and_version(tool_name: &str, version: VersionSpec) -> Self {
        match tool_name {
            "node" => ToolSpec::Node(version),
            "npm" => ToolSpec::Npm(version),
            "yarn" => ToolSpec::Yarn(version),
            package => ToolSpec::Package(package.to_string(), version),
        }
    }

    pub fn install(&self, session: &mut Session) -> Fallible<()> {
        match self {
            ToolSpec::Node(version) => session.install_node(&version)?,
            // ISSUE(#292): Implement install for npm
            ToolSpec::Npm(_version) => unimplemented!("Installing npm is not supported yet"),
            ToolSpec::Yarn(version) => session.install_yarn(&version)?,
            ToolSpec::Package(name, version) => {
                session.install_package(name.to_string(), &version)?;
            }
        }
        Ok(())
    }

    pub fn uninstall(&self, session: &mut Session) -> Fallible<()> {
        match self {
            ToolSpec::Node(_version) => unimplemented!("Uninstalling Node not supported yet"),
            // ISSUE(#292): Implement install for npm
            ToolSpec::Npm(_version) => unimplemented!("Uninstalling Npm not supported yet"),
            ToolSpec::Yarn(_version) => unimplemented!("Uninstalling Yarn not supported yet"),
            ToolSpec::Package(name, _version) => {
                session.uninstall_package(name.to_string())?;
            }
        }
        Ok(())
    }

    /// Try to parse a tool and version from a string like `<tool>[@<version>].
    pub fn try_from_str(tool_spec: &str) -> Fallible<Self> {
        let captures =
            TOOL_SPEC_PATTERN
                .captures(tool_spec)
                .ok_or(ErrorDetails::ParseToolSpecError {
                    tool_spec: tool_spec.into(),
                })?;

        // Validate that the captured name is a valid NPM package name.
        let name = &captures["name"];
        if let Validity::Invalid { errors, .. } = validate(name) {
            return Err(ErrorDetails::InvalidToolName {
                name: name.into(),
                errors,
            }
            .into());
        }

        let version = captures
            .name("version")
            .map(|version| VersionSpec::parse(version.as_str()))
            .transpose()?
            .unwrap_or_default();

        Ok(match name {
            "node" => ToolSpec::Node(version),
            "npm" => ToolSpec::Npm(version),
            "yarn" => ToolSpec::Yarn(version),
            package => ToolSpec::Package(package.into(), version),
        })
    }
}

impl Debug for ToolSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            &ToolSpec::Node(ref version) => format!("node version {}", version),
            &ToolSpec::Yarn(ref version) => format!("yarn version {}", version),
            &ToolSpec::Npm(ref version) => format!("npm version {}", version),
            &ToolSpec::Package(ref name, ref version) => format!("{} version {}", name, version),
        };
        f.write_str(&s)
    }
}

impl Display for ToolSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            &ToolSpec::Node(ref version) => format!("node version {}", version),
            &ToolSpec::Yarn(ref version) => format!("yarn version {}", version),
            &ToolSpec::Npm(ref version) => format!("npm version {}", version),
            &ToolSpec::Package(ref name, ref version) => format!("{} version {}", name, version),
        };
        f.write_str(&s)
    }
}

pub fn execute_tool(session: &mut Session) -> Fallible<ExitStatus> {
    path::ensure_notion_dirs_exist()?;

    let mut args = args_os();
    let exe = get_tool_name(&mut args)?;

    let command = match &exe.to_str() {
        Some("node") => node::command(args, session)?,
        Some("npm") => npm::command(args, session)?,
        Some("npx") => npx::command(args, session)?,
        Some("yarn") => yarn::command(args, session)?,
        _ => binary::command(exe, args, session)?,
    };

    command.exec()
}

/// Represents the command to execute a tool
enum ToolCommand {
    Direct(Command),
    Passthrough(Command, ErrorDetails),
}

impl ToolCommand {
    fn direct<A>(exe: &OsStr, args: A, path_var: &OsStr) -> Self
    where
        A: IntoIterator<Item = OsString>,
    {
        ToolCommand::Direct(command_for(exe, args, path_var))
    }

    fn passthrough<A>(exe: &OsStr, args: A, default_error: ErrorDetails) -> Fallible<Self>
    where
        A: IntoIterator<Item = OsString>,
    {
        let path = System::path()?;
        Ok(ToolCommand::Passthrough(
            command_for(exe, args, &path),
            default_error,
        ))
    }

    fn exec(self) -> Fallible<ExitStatus> {
        match self {
            ToolCommand::Direct(mut command) => command
                .status()
                .with_context(|_| ErrorDetails::BinaryExecError),
            ToolCommand::Passthrough(mut command, error) => {
                command.status().with_context(|_| error)
            }
        }
    }
}

fn get_tool_name(args: &mut ArgsOs) -> Fallible<OsString> {
    args.nth(0)
        .and_then(|arg0| Path::new(&arg0).file_name().map(tool_name_from_file_name))
        .ok_or(ErrorDetails::CouldNotDetermineTool.into())
}

#[cfg(unix)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    file_name.to_os_string()
}

#[cfg(windows)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    // On Windows PowerShell, the file name includes the .exe suffix
    // We need to remove that to get the raw tool name
    match file_name.to_str() {
        Some(file) => OsString::from(file.trim_end_matches(".exe")),
        None => OsString::from(file_name),
    }
}

fn command_for<A>(exe: &OsStr, args: A, path_var: &OsStr) -> Command
where
    A: IntoIterator<Item = OsString>,
{
    let mut command = create_command(exe);
    command.args(args);
    command.env("PATH", path_var);
    command
}

fn intercept_global_installs() -> bool {
    // We should only intercept global installs if the NOTION_UNSAFE_GLOBAL variable is not set
    env::var_os(UNSAFE_GLOBAL).is_none()
}

#[cfg(test)]
mod tests {
    mod try_from_str {
        use std::str::FromStr as _;

        use super::super::ToolSpec;
        use crate::version::VersionSpec;

        const LTS: &str = "lts";
        const LATEST: &str = "latest";
        const MAJOR: &str = "3";
        const MINOR: &str = "3.0";
        const PATCH: &str = "3.0.0";

        /// Convenience macro for generating the <tool>@<version> string.
        macro_rules! versioned_tool {
            ($tool:expr, $version:expr) => {
                format!("{}@{}", $tool, $version)
            };
        }

        #[test]
        fn parses_bare_node() {
            assert_eq!(
                ToolSpec::try_from_str("node").expect("succeeds"),
                ToolSpec::Node(VersionSpec::default())
            );
        }

        #[test]
        fn parses_node_with_valid_versions() {
            let tool = "node";

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, MAJOR)).expect("succeeds"),
                ToolSpec::Node(
                    VersionSpec::from_str(MAJOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, MINOR)).expect("succeeds"),
                ToolSpec::Node(
                    VersionSpec::from_str(MINOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, PATCH)).expect("succeeds"),
                ToolSpec::Node(
                    VersionSpec::from_str(PATCH).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, LATEST)).expect("succeeds"),
                ToolSpec::Node(VersionSpec::Latest)
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, LTS)).expect("succeeds"),
                ToolSpec::Node(VersionSpec::Lts)
            );
        }

        #[test]
        fn parses_bare_yarn() {
            assert_eq!(
                ToolSpec::try_from_str("yarn").expect("succeeds"),
                ToolSpec::Yarn(VersionSpec::default())
            );
        }

        #[test]
        fn parses_yarn_with_valid_versions() {
            let tool = "yarn";

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, MAJOR)).expect("succeeds"),
                ToolSpec::Yarn(
                    VersionSpec::from_str(MAJOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, MINOR)).expect("succeeds"),
                ToolSpec::Yarn(
                    VersionSpec::from_str(MINOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, PATCH)).expect("succeeds"),
                ToolSpec::Yarn(
                    VersionSpec::from_str(PATCH).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, LATEST)).expect("succeeds"),
                ToolSpec::Yarn(VersionSpec::Latest)
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, LTS)).expect("succeeds"),
                ToolSpec::Yarn(VersionSpec::Lts)
            );
        }

        #[test]
        fn parses_bare_packages() {
            let package = "ember-cli";
            assert_eq!(
                ToolSpec::try_from_str(package).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::default())
            );
        }

        #[test]
        fn parses_namespaced_packages() {
            let package = "@types/lodash";
            assert_eq!(
                ToolSpec::try_from_str(package).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::default())
            );
        }

        #[test]
        fn parses_bare_packages_with_valid_versions() {
            let package = "something-awesome";

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, MAJOR)).expect("succeeds"),
                ToolSpec::Package(
                    package.into(),
                    VersionSpec::from_str(MAJOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, MINOR)).expect("succeeds"),
                ToolSpec::Package(
                    package.into(),
                    VersionSpec::from_str(MINOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, PATCH)).expect("succeeds"),
                ToolSpec::Package(
                    package.into(),
                    VersionSpec::from_str(PATCH).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, LATEST)).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::Latest)
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, LTS)).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::Lts)
            );
        }

        #[test]
        fn parses_namespaced_packages_with_valid_versions() {
            let package = "@something/awesome";

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, MAJOR)).expect("succeeds"),
                ToolSpec::Package(
                    package.into(),
                    VersionSpec::from_str(MAJOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, MINOR)).expect("succeeds"),
                ToolSpec::Package(
                    package.into(),
                    VersionSpec::from_str(MINOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, PATCH)).expect("succeeds"),
                ToolSpec::Package(
                    package.into(),
                    VersionSpec::from_str(PATCH).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, LATEST)).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::Latest)
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, LTS)).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::Lts)
            );
        }
    }
}
