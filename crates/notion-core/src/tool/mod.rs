//! Traits and types for executing command-line tools.

use std::env::{self, args_os, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Debug, Display, Formatter};
use std::marker::Sized;
use std::path::Path;
use std::process::{Command, ExitStatus};
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;

use notion_fail::{throw, Fallible, ResultExt};
use validate_npm_package_name::{validate, Validity};

use crate::command::create_command;
use crate::env::UNSAFE_GLOBAL;
use crate::error::ErrorDetails;
use crate::path;
use crate::session::Session;
use crate::version::VersionSpec;

mod binary;
mod node;
mod npm;
mod npx;
mod yarn;

use self::binary::{Binary, BinaryArgs};
use self::node::Node;
use self::npm::Npm;
use self::npx::Npx;
use self::yarn::Yarn;

lazy_static! {
    static ref TOOL_SPEC_PATTERN: Regex =
        Regex::new("^(?P<name>(?:@([^/]+?)[/])?([^/]+?))(@(?P<version>.+))?$")
            .expect("regex is valid");
    static ref HAS_VERSION: Regex = Regex::new(r"^[^\s]+@").expect("regex is valid");
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
            ToolSpec::Npm(_version) => throw!(ErrorDetails::Unimplemented {
                feature: "Installing npm".into()
            }),
            ToolSpec::Yarn(version) => session.install_yarn(&version)?,
            ToolSpec::Package(name, version) => {
                session.install_package(name.to_string(), &version)?;
            }
        };
        Ok(())
    }

    pub fn uninstall(&self, session: &mut Session) -> Fallible<()> {
        match self {
            ToolSpec::Node(_version) => throw!(ErrorDetails::Unimplemented {
                feature: "Uninstalling node".into()
            }),
            // ISSUE(#292): Implement install for npm
            ToolSpec::Npm(_version) => throw!(ErrorDetails::Unimplemented {
                feature: "Uninstalling npm".into()
            }),
            ToolSpec::Yarn(_version) => throw!(ErrorDetails::Unimplemented {
                feature: "Uninstalling yarn".into()
            }),
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

    /// Get a valid, sorted `Vec<ToolSpec>` given a `Vec<String>`.
    ///
    /// Accounts for the following error conditions:
    ///
    /// - `notion install node 12`, where the user intended to install `node@12`
    ///   but used syntax like in nodenv or nvm
    /// - invalid version specs
    ///
    /// Returns a listed sorted so that if `node` is included in the list, it is
    /// always first.
    pub fn from_strings<T>(tool_strs: &mut [T], action: &str) -> Fallible<Vec<ToolSpec>>
    where
        T: AsRef<str>,
    {
        Self::check_args(tool_strs, action)?;

        let mut tools = tool_strs
            .iter()
            .map(|arg| Self::try_from_str(arg.as_ref()))
            .collect::<Fallible<Vec<ToolSpec>>>()?;

        tools.sort();
        Ok(tools)
    }

    /// Check the args for the bad pattern of `notion install <tool> <number>`.
    fn check_args<T>(args: &mut [T], action: &str) -> Fallible<()>
    where
        T: AsRef<str>,
    {
        let mut args = args.iter();

        // The case we are concerned with is where we have `<tool> <number>`.
        // This is only interesting if there are exactly two args. Then we care
        // whether the two items are a bare name (with no `@version`), followed
        // by a valid version specifier. That is:
        //
        // - `notion install node@lts latest` is allowed.
        // - `notion install node latest` is an error.
        // - `notion install node latest yarn` is allowed.
        if let (Some(name), Some(maybe_version), None) = (args.next(), args.next(), args.next()) {
            if !HAS_VERSION.is_match(name.as_ref())
                && VersionSpec::from_str(maybe_version.as_ref()).is_ok()
            {
                return Err(ErrorDetails::InvalidInvocation {
                    action: action.to_string(),
                    name: name.as_ref().to_string(),
                    version: maybe_version.as_ref().to_string(),
                }
                .into());
            }
        }

        Ok(())
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

    // There is some duplication in the calls to `.exec` here.
    // It's required because we can't create a single variable that holds
    // all the possible `Tool` implementations and fill it dynamically,
    // as they have different sizes and associated types.
    match &exe.to_str() {
        Some("node") => Node::new(args, session)?.exec(),
        Some("npm") => Npm::new(args, session)?.exec(),
        Some("npx") => Npx::new(args, session)?.exec(),
        Some("yarn") => Yarn::new(args, session)?.exec(),
        _ => Binary::new(
            BinaryArgs {
                executable: exe,
                args,
            },
            session,
        )?
        .exec(),
    }
}

/// Represents a command-line tool that Notion shims delegate to.
pub trait Tool: Sized {
    type Arguments;

    /// Constructs a new instance.
    fn new(args: Self::Arguments, session: &mut Session) -> Fallible<Self>;

    /// Extracts the `Command` from this tool.
    fn command(self) -> Command;

    /// Delegates the current process to this tool.
    fn exec(self) -> Fallible<ExitStatus> {
        let mut command = self.command();
        let status = command.status();
        status.with_context(|_| ErrorDetails::BinaryExecError)
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
    A: Iterator<Item = OsString>,
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

    mod from_strings {
        use super::super::*;

        static PIN: &'static str = "pin";

        #[test]
        fn special_cases_tool_space_number() {
            let name = "potato";
            let version = "1.2.3";
            let mut args: Vec<String> = vec![name.into(), version.into()];

            let err = ToolSpec::from_strings(&mut args, PIN).unwrap_err();
            let inner_err = err
                .downcast_ref::<ErrorDetails>()
                .expect("should be an ErrorDetails");

            assert_eq!(
                inner_err,
                &ErrorDetails::InvalidInvocation {
                    action: PIN.into(),
                    name: name.into(),
                    version: version.into()
                },
                "`notion <action> tool number` results in the correct error"
            );
        }

        #[test]
        fn leaves_other_scenarios_alone() {
            let mut empty: Vec<&str> = Vec::new();
            assert!(
                ToolSpec::from_strings(&mut empty, PIN).is_ok(),
                "when there are no args"
            );

            assert!(
                ToolSpec::from_strings(&mut ["node".to_owned()], PIN).is_ok(),
                "when there is only one arg"
            );

            assert!(
                ToolSpec::from_strings(&mut ["12".to_owned(), "node".to_owned()], PIN.into())
                    .is_ok(),
                "when there are two args but the order is not likely to be a mistake"
            );

            assert!(
                ToolSpec::from_strings(&mut ["node@lts".to_owned(), "12".to_owned()], PIN.into())
                    .is_ok(),
                "when there are two args but the first is a valid tool spec"
            );

            assert!(
                ToolSpec::from_strings(
                    &mut ["node".to_owned(), "12".to_owned(), "yarn".to_owned(),],
                    PIN.into()
                )
                .is_ok(),
                "when there are more than two args"
            );
        }
    }
}
