use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::PathBuf;

use console::style;
use notion_core::project::Project;
use notion_core::session::{ActivityKind, Session};
use notion_core::{path, shim};
use notion_fail::{ExitCode, Fallible, ResultExt};
use semver::Version;

use Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_shimname: Option<String>,
    flag_delete: bool,
    flag_verbose: bool,
}

pub(crate) enum Shim {
    Help,
    List(bool),
    Create(String, bool),
    Delete(String, bool),
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
    notion shim [options]
    notion shim <shimname> [options]

Options:
    -d, --delete   Delete 3rd-party shim
    -v, --verbose  Verbose output
    -h, --help     Display this message

";

    fn help() -> Self {
        Shim::Help
    }

    fn parse(
        _: Notion,
        Args {
            arg_shimname,
            flag_delete,
            flag_verbose,
        }: Args,
    ) -> Fallible<Self> {
        Ok(if let Some(shim_name) = arg_shimname {
            if flag_delete {
                Shim::Delete(shim_name, flag_verbose)
            } else {
                Shim::Create(shim_name, flag_verbose)
            }
        } else {
            Shim::List(flag_verbose)
        })
    }

    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Shim);

        match self {
            Shim::Help => Help::Command(CommandName::Shim).run(session)?,
            Shim::List(verbose) => list(session, verbose)?,
            Shim::Create(shim_name, verbose) => create(session, shim_name, verbose)?,
            Shim::Delete(shim_name, verbose) => delete(session, shim_name, verbose)?,
        };
        session.add_event_end(ActivityKind::Shim, ExitCode::Success);
        Ok(())
    }
}

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

fn resolve_shim(session: &Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    match shim_name.to_str() {
        Some("node") | Some("npm") => resolve_node_shims(session, shim_name),
        Some("yarn") => resolve_yarn_shims(session, shim_name),
        Some("npx") => resolve_npx_shims(session, shim_name),
        Some(_) => resolve_3p_shims(session, shim_name),
        None => panic!("Cannot format {} as a string", shim_name.to_string_lossy()),
    }
}

fn is_node_version_installed(project: &Project, session: &Session) -> Fallible<bool> {
    let catalog = session.catalog()?;
    Ok(catalog.node.contains(&project.manifest().node().unwrap()))
}

// figure out which version of Node is installed or configured,
// or which version will be installed if it's not pinned by the project
fn resolve_node_shims(session: &Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    if session.in_pinned_project() {
        let project = session.project().unwrap();
        let version = &project.manifest().node().unwrap();
        if is_node_version_installed(&project, &session)? {
            // Node is pinned by the project - this shim will use that version
            let mut bin_path = path::node_version_bin_dir(&version.to_string()).unknown()?;
            bin_path.push(&shim_name);
            return Ok(ShimKind::User(bin_path));
        }

        // not installed, but will install based on the required version
        return Ok(ShimKind::WillInstall(version.clone()));
    }

    if let Some(user_version) = session.user_node()? {
        let mut bin_path = path::node_version_bin_dir(&user_version.to_string()).unknown()?;
        bin_path.push(&shim_name);
        return Ok(ShimKind::User(bin_path));
    }
    Ok(ShimKind::System)
}

fn resolve_yarn_shims(session: &Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    if session.in_pinned_project() {
        let project = session.project().unwrap();
        if let Some(ref version) = &project.manifest().yarn() {
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
    if session.in_pinned_project() {
        let project = session.project().unwrap();
        // if this is a local executable, get the path to that
        if project.has_direct_bin(shim_name)? {
            let mut path_to_bin = project.local_bin_dir();
            path_to_bin.push(shim_name);
            return Ok(ShimKind::Project(path_to_bin));
        }

        // if Node is installed, use the bin there
        if is_node_version_installed(&project, &session)? {
            let version = project.manifest().node().unwrap();
            // Node is pinned by the project - this shim will use that version
            let mut bin_path = path::node_version_3p_bin_dir(&version.to_string())?;
            bin_path.push(&shim_name);
            return Ok(ShimKind::User(bin_path));
        }
        // if Node is not installed, this shim has not been installed for this node version
        return Ok(ShimKind::NotInstalled);
    }
    // if a user Node is configured with Notion, use that executable
    // otherwise it's a shim to system executables
    let user_version = session.user_node()?;
    user_version.map_or(Ok(ShimKind::System), |gv| {
        let mut bin_path = path::node_version_3p_bin_dir(&gv.to_string())?;
        bin_path.push(&shim_name);
        Ok(ShimKind::User(bin_path))
    })
}
