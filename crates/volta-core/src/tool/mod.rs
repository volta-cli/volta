//! Traits and types for executing command-line tools.

use std::cmp::Ordering;
use std::env::{self, args_os, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Debug, Display, Formatter};
use std::path::Path;
use std::process::{Command, ExitStatus};
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;

use validate_npm_package_name::{validate, Validity};
use volta_fail::{throw, Fallible, ResultExt};

use crate::command::create_command;
use crate::env::UNSAFE_GLOBAL;
use crate::error::ErrorDetails;
use crate::path;
use crate::platform::System;
use crate::session::Session;
use crate::signal::pass_control_to_shim;
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
    static ref HAS_VERSION: Regex = Regex::new(r"^[^\s]+@").expect("regex is valid");
}

/// Distinguish global `add` commands in npm or yarn from all others.
enum CommandArg {
    /// The command is a *global* add command.
    GlobalAdd(Option<OsString>),
    /// The command is a local, i.e. non-global, add command.
    NotGlobalAdd,
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
#[derive(PartialEq)]
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
    /// - `volta install node 12`, where the user intended to install `node@12`
    ///   but used syntax like in nodenv or nvm
    /// - invalid version specs
    ///
    /// Returns a listed sorted so that if `node` is included in the list, it is
    /// always first.
    pub fn from_strings<T>(tool_strs: &[T], action: &str) -> Fallible<Vec<ToolSpec>>
    where
        T: AsRef<str>,
    {
        Self::check_args(tool_strs, action)?;

        let mut tools = tool_strs
            .iter()
            .map(|arg| Self::try_from_str(arg.as_ref()))
            .collect::<Fallible<Vec<ToolSpec>>>()?;

        tools.sort_by(Self::sort_comparator);
        Ok(tools)
    }

    /// Check the args for the bad pattern of `volta install <tool> <number>`.
    fn check_args<T>(args: &[T], action: &str) -> Fallible<()>
    where
        T: AsRef<str>,
    {
        let mut args = args.iter();

        // The case we are concerned with is where we have `<tool> <number>`.
        // This is only interesting if there are exactly two args. Then we care
        // whether the two items are a bare name (with no `@version`), followed
        // by a valid version specifier. That is:
        //
        // - `volta install node@lts latest` is allowed.
        // - `volta install node latest` is an error.
        // - `volta install node latest yarn` is allowed.
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

    /// Compare `ToolSpec`s for sorting when converting from strings
    ///
    /// We want to preserve the original order as much as possible, so we treat tools in
    /// the same tool category as equal. We still need to pull Node to the front of the
    /// list, followed by Npm / Yarn, and then Packages last.
    fn sort_comparator(left: &ToolSpec, right: &ToolSpec) -> Ordering {
        match (left, right) {
            (ToolSpec::Node(_), ToolSpec::Node(_)) => Ordering::Equal,
            (ToolSpec::Node(_), _) => Ordering::Less,
            (_, ToolSpec::Node(_)) => Ordering::Greater,
            (ToolSpec::Npm(_), ToolSpec::Npm(_)) => Ordering::Equal,
            (ToolSpec::Npm(_), _) => Ordering::Less,
            (_, ToolSpec::Npm(_)) => Ordering::Greater,
            (ToolSpec::Yarn(_), ToolSpec::Yarn(_)) => Ordering::Equal,
            (ToolSpec::Yarn(_), _) => Ordering::Less,
            (_, ToolSpec::Yarn(_)) => Ordering::Greater,
            (ToolSpec::Package(_, _), ToolSpec::Package(_, _)) => Ordering::Equal,
        }
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
    path::ensure_volta_dirs_exist()?;

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
struct ToolCommand {
    command: Command,
    error: ErrorDetails,
}

impl ToolCommand {
    fn direct<A>(exe: &OsStr, args: A, path_var: &OsStr) -> Self
    where
        A: IntoIterator<Item = OsString>,
    {
        ToolCommand {
            command: command_for(exe, args, path_var),
            error: ErrorDetails::BinaryExecError,
        }
    }

    fn project_local<A>(exe: &OsStr, args: A, path_var: &OsStr) -> Self
    where
        A: IntoIterator<Item = OsString>,
    {
        ToolCommand {
            command: command_for(exe, args, path_var),
            error: ErrorDetails::ProjectLocalBinaryExecError {
                command: exe.to_string_lossy().to_string(),
            },
        }
    }

    fn passthrough<A>(exe: &OsStr, args: A, default_error: ErrorDetails) -> Fallible<Self>
    where
        A: IntoIterator<Item = OsString>,
    {
        let path = System::path()?;
        Ok(ToolCommand {
            command: command_for(exe, args, &path),
            error: default_error,
        })
    }

    fn exec(mut self) -> Fallible<ExitStatus> {
        pass_control_to_shim();
        self.command.status().with_context(|_| self.error)
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
    // We should only intercept global installs if the VOLTA_UNSAFE_GLOBAL variable is not set
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
            let args: Vec<String> = vec![name.into(), version.into()];

            let err = ToolSpec::from_strings(&args, PIN).unwrap_err();
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
                "`volta <action> tool number` results in the correct error"
            );
        }

        #[test]
        fn leaves_other_scenarios_alone() {
            let empty: Vec<&str> = Vec::new();
            assert_eq!(
                ToolSpec::from_strings(&empty, PIN).expect("is ok").len(),
                empty.len(),
                "when there are no args"
            );

            let only_one = ["node".to_owned()];
            assert_eq!(
                ToolSpec::from_strings(&only_one, PIN).expect("is ok").len(),
                only_one.len(),
                "when there is only one arg"
            );

            let two_but_unmistakable = ["12".to_owned(), "node".to_owned()];
            assert_eq!(
                ToolSpec::from_strings(&two_but_unmistakable, PIN.into())
                    .expect("is ok")
                    .len(),
                two_but_unmistakable.len(),
                "when there are two args but the order is not likely to be a mistake"
            );

            let two_but_valid_first = ["node@lts".to_owned(), "12".to_owned()];
            assert_eq!(
                ToolSpec::from_strings(&two_but_valid_first, PIN.into())
                    .expect("is ok")
                    .len(),
                two_but_valid_first.len(),
                "when there are two args but the first is a valid tool spec"
            );

            let more_than_two_tools = ["node".to_owned(), "12".to_owned(), "yarn".to_owned()];
            assert_eq!(
                ToolSpec::from_strings(&more_than_two_tools, PIN.into())
                    .expect("is ok")
                    .len(),
                more_than_two_tools.len(),
                "when there are more than two args"
            );
        }

        #[test]
        fn sorts_node_npm_yarn_to_front() {
            let multiple = [
                "ember-cli@3".to_owned(),
                "yarn".to_owned(),
                "npm@5".to_owned(),
                "node@latest".to_owned(),
            ];
            let expected = [
                ToolSpec::Node(VersionSpec::Latest),
                ToolSpec::Npm(VersionSpec::from_str("5").expect("requirement is valid")),
                ToolSpec::Yarn(VersionSpec::default()),
                ToolSpec::Package(
                    "ember-cli".to_owned(),
                    VersionSpec::from_str("3").expect("requirement is valid"),
                ),
            ];
            assert_eq!(
                ToolSpec::from_strings(&multiple, PIN.into()).expect("is ok"),
                expected
            );
        }

        #[test]
        fn keeps_package_order_unchanged() {
            let packages_with_node = ["typescript@latest", "ember-cli@3", "node@lts", "mocha"];
            let expected = [
                ToolSpec::Node(VersionSpec::Lts),
                ToolSpec::Package("typescript".to_owned(), VersionSpec::Latest),
                ToolSpec::Package(
                    "ember-cli".to_owned(),
                    VersionSpec::from_str("3").expect("requirement is valid"),
                ),
                ToolSpec::Package("mocha".to_owned(), VersionSpec::default()),
            ];

            assert_eq!(
                ToolSpec::from_strings(&packages_with_node, PIN.into()).expect("is ok"),
                expected
            );
        }
    }
}
