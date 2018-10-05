use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::PathBuf;

use console::style;
use notion_core::project::Project;
use notion_core::session::{ActivityKind, Session};
use notion_core::style::{display_error, display_unknown_error, ErrorContext};
use notion_core::{path, shim};
use notion_fail::{ExitCode, Fallible, NotionFail, ResultExt};
use semver::Version;

use Notion;
use command::{Command, CommandName, Help};

/// Thrown when one or more errors occurred while autoshimming.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "auto shimming did not complete without failures")]
#[notion_fail(code = "UnknownError")]
struct AutoshimError;

/// Thrown when the user tries to create or delete a shim without specifying a name.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "you must specify a shim name")]
#[notion_fail(code = "InvalidArguments")]
struct MissingShimNameError;

/// Thrown when the user tries to autoshim outside of a Node package without supplying
/// a target directory.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "{} is not a node package", path)]
#[notion_fail(code = "ConfigurationError")]
struct NotAPackageError {
    path: String,
}

/// Thrown when the user supplies a shim name with the "list" subcommand.alloc
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "the list command takes no additional arguments")]
#[notion_fail(code = "InvalidArguments")]
struct PresentShimNameError;

/// Thrown when the user supplies an unrecognized subcommand.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "unrecognized shim command {}", command)]
#[notion_fail(code = "InvalidArguments")]
struct UnrecognizedCommandError {
    command: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_command: Option<String>,
    arg_shim_name_or_path: Option<String>,
    flag_verbose: bool,
}

pub(crate) enum Shim {
    Help,
    List(bool),
    Create(String, bool),
    Delete(String, bool),
    Auto(Option<PathBuf>, bool),
}

enum ShimKind {
    Project(PathBuf),
    User(PathBuf),
    System,
    NotInstalled,
    WillInstall(Version),
    Unimplemented,
}

impl Display for ShimKind {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            &ShimKind::Project(ref path) => format!("{}", path.to_string_lossy()),
            &ShimKind::User(ref path) => format!("{}", path.to_string_lossy()),
            &ShimKind::System => format!("[system]"),
            &ShimKind::NotInstalled => {
                format!("{}", style("[executable not installed!]").red().bold())
            }
            &ShimKind::WillInstall(ref version) => format!("[will install version {}]", version),
            &ShimKind::Unimplemented => {
                format!("{}", style("[shim not implemented!]").red().bold())
            }
        };
        f.write_str(&s)
    }
}

impl Command for Shim {
    type Args = Args;

    const USAGE: &'static str = "
Manage Notion shims for 3rd-party executables

Usage:
    notion shim list [options]
    notion shim create <shim name> [options]
    notion shim delete <shim name> [options]
    notion shim auto [path] [options]

Options:
    -v, --verbose  Verbose output
    -h, --help     Display this message

";

    fn help() -> Self {
        Shim::Help
    }

    fn parse(
        _: Notion,
        Args {
            arg_command,
            arg_shim_name_or_path,
            flag_verbose,
        }: Args,
    ) -> Fallible<Self> {
        Ok(match arg_command {
            Some(ref command) if command == &"auto".to_string() => {
                if let Some(path_string) = arg_shim_name_or_path {
                    Shim::Auto(Some(PathBuf::from(path_string)), flag_verbose)
                } else {
                    Shim::Auto(None, flag_verbose)
                }
            },
            Some(ref command) if command == &"create".to_string() => {
                if let Some(shim_name) = arg_shim_name_or_path {
                    Shim::Create(shim_name, flag_verbose)
                } else {
                    throw!(MissingShimNameError);
                }
            },
            Some(ref command) if command == &"delete".to_string() => {
                if let Some(shim_name) = arg_shim_name_or_path {
                    Shim::Delete(shim_name, flag_verbose)
                } else {
                    throw!(MissingShimNameError);
                }
            },
            Some(ref command) if command == &"list".to_string() => {
                if let Some(_) = arg_shim_name_or_path {
                    throw!(PresentShimNameError);
                } else {
                    Shim::List(flag_verbose)
                }
            },
            Some(command) => throw!(UnrecognizedCommandError {
                command: command,
            }),
            None => Shim::Help,
        })
    }

    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Shim);

        match self {
            Shim::Help => Help::Command(CommandName::Shim).run(session)?,
            Shim::List(verbose) => list(session, verbose)?,
            Shim::Create(shim_name, verbose) => create(session, shim_name, verbose)?,
            Shim::Delete(shim_name, verbose) => delete(session, shim_name, verbose)?,
            Shim::Auto(path, verbose) => autoshim(session, path, verbose)?,
        };
        session.add_event_end(ActivityKind::Shim, ExitCode::Success);
        Ok(())
    }
}

// ISSUE(#143): all the logic for this should be moved to notion-core
fn list(session: &Session, verbose: bool) -> Fallible<()> {
    let shim_dir = path::shim_dir()?;
    let files = fs::read_dir(shim_dir).unknown()?;

    for file in files {
        let file = file.unknown()?;
        print_file_info(file, session, verbose)?;
    }
    Ok(())
}

fn print_file_info(file: fs::DirEntry, session: &Session, verbose: bool) -> Fallible<()> {
    let shim_name = file.file_name();
    if verbose {
        let shim_info = resolve_shim(session, &shim_name)?;
        println!("{} -> {}", shim_name.to_string_lossy(), shim_info);
    } else {
        println!("{}", shim_name.to_string_lossy());
    }
    Ok(())
}

fn create(_session: &Session, shim_name: String, _verbose: bool) -> Fallible<()> {
    shim::create(&shim_name)?;
    Ok(())
}

fn delete(_session: &Session, shim_name: String, _verbose: bool) -> Fallible<()> {
    shim::delete(&shim_name)?;
    Ok(())
}

fn autoshim(session: &Session, maybe_path: Option<PathBuf>, _verbose: bool) -> Fallible<()> {
    let errors = if let Some(path) = maybe_path {
        if let Some(path_project) = Project::for_dir(&path)? {
            path_project.autoshim()
        } else {
            throw!(NotAPackageError {
                path: path.to_str().unwrap().to_string(),
            })
        }
    } else if let Some(session_project) = session.project() {
        session_project.autoshim()
    } else {
        throw!(NotAPackageError {
            path: ".".to_string(),
        })
    };

    if errors.len() == 0 {
        Ok(())
    } else {
        for error in errors {
            if error.is_user_friendly() {
                display_error(ErrorContext::Notion, &error);
            } else {
                display_unknown_error(ErrorContext::Notion, &error);
            }
        }

        throw!(AutoshimError)
    }
}

fn resolve_shim(session: &Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    match shim_name.to_str() {
        Some("node") | Some("npm") => resolve_node_shims(session, shim_name),
        Some("yarn") => resolve_yarn_shims(session, shim_name),
        Some("npx") => resolve_npx_shims(session, shim_name),
        Some(_) => resolve_3p_shims(session, shim_name),
        None => panic!("Cannot format {} as a string", shim_name.to_string_lossy()),
    }
}

fn is_node_version_installed(version: &Version, session: &Session) -> Fallible<bool> {
    Ok(session.catalog()?.node.contains(version))
}

// figure out which version of Node is installed or configured,
// or which version will be installed if it's not pinned by the project
fn resolve_node_shims(session: &Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    if let Some(ref image) = session.project_platform() {
        if is_node_version_installed(&image.node, &session)? {
            // Node is pinned by the project - this shim will use that version
            let mut bin_path = path::node_version_bin_dir(&image.node_str).unknown()?;
            bin_path.push(&shim_name);
            return Ok(ShimKind::User(bin_path));
        }

        return Ok(ShimKind::WillInstall(image.node.clone()));
    }

    if let Some(user_version) = session.user_node()? {
        let mut bin_path = path::node_version_bin_dir(&user_version.to_string()).unknown()?;
        bin_path.push(&shim_name);
        return Ok(ShimKind::User(bin_path));
    }
    Ok(ShimKind::System)
}

fn resolve_yarn_shims(session: &Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    if let Some(ref image) = session.project_platform() {
        if let Some(ref version) = image.yarn {
            let catalog = session.catalog()?;
            if catalog.yarn.contains(version) {
                // Yarn is pinned by the project - this shim will use that version
                let mut bin_path = path::yarn_version_bin_dir(&version.to_string()).unknown()?;
                bin_path.push(&shim_name);
                return Ok(ShimKind::User(bin_path));
            }

            // not installed, but will install based on the required version
            return Ok(ShimKind::WillInstall(version.clone()));
        }

        return Ok(ShimKind::NotInstalled);
    }

    if let Some(ref default_version) = session.catalog()?.yarn.default {
        let mut bin_path = path::yarn_version_bin_dir(&default_version.to_string()).unknown()?;
        bin_path.push(&shim_name);
        return Ok(ShimKind::User(bin_path));
    }
    Ok(ShimKind::System)
}

fn resolve_npx_shims(_session: &Session, _shim_name: &OsStr) -> Fallible<ShimKind> {
    Ok(ShimKind::Unimplemented)
}

fn resolve_3p_shims(session: &Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    if let Some(ref project) = session.project() {
        // if this is a local executable, get the path to that
        if project.has_direct_bin(shim_name)? {
            let mut path_to_bin = project.local_bin_dir();
            path_to_bin.push(shim_name);
            return Ok(ShimKind::Project(path_to_bin));
        }
    }
    Ok(ShimKind::NotInstalled)
}
