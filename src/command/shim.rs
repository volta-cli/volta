use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::PathBuf;

use notion_core::session::{ActivityKind, Session};
use notion_core::{path, style};
use notion_fail::{Fallible, ResultExt};

use Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_shimname: Option<String>,
    flag_verbose: bool,
}

pub(crate) enum Shim {
    Help,
    List(bool),
    Create(String, bool),
}

enum ShimKind {
    Local(PathBuf),
    Global(PathBuf),
    System,
    Unimplemented,
}

impl Display for ShimKind {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            &ShimKind::Local(ref path) => format!("{}", path.to_string_lossy()),
            &ShimKind::Global(ref path) => format!("{}", path.to_string_lossy()),
            &ShimKind::System => format!("[system]"),
            &ShimKind::Unimplemented => format!("[shim not implemented!]"),
        };
        f.write_str(&s)
    }
}

impl Command for Shim {
    type Args = Args;

    const USAGE: &'static str = "
Manage Notion shims for 3rd-party executables

Usage:
    notion shim [<shimname>] [options]

Options:
    -v, --verbose  Verbose output
    -h, --help     Display this message

";
    // TODO more verbiage about how to use this command

    fn help() -> Self {
        Shim::Help
    }

    fn parse(
        _: Notion,
        Args {
            arg_shimname,
            flag_verbose,
        }: Args,
    ) -> Fallible<Self> {
        Ok(if let Some(shim_name) = arg_shimname {
            Shim::Create(shim_name, flag_verbose)
        } else {
            Shim::List(flag_verbose)
        })
    }

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Shim);

        let result = match self {
            Shim::Help => Help::Command(CommandName::Shim).run(session),
            Shim::List(verbose) => list(session, verbose),
            Shim::Create(shim_name, verbose) => create(session, shim_name, verbose),
        };
        session.add_event_end(ActivityKind::Shim, 0);
        result
    }
}

fn list(session: &mut Session, verbose: bool) -> Fallible<bool> {
    path::shim_dir()
        .and_then(|shim_dir| fs::read_dir(shim_dir).unknown())
        .map(|files| {
            files
                .map(|file| {
                    file.and_then(|f| {
                        f.path().file_name().map_or(Ok(true), |shim_name| {
                            if verbose {
                                match resolve_shim(session, &shim_name) {
                                    Ok(shim_info) => {
                                        println!("{} -> {}", shim_name.to_string_lossy(), shim_info)
                                    }
                                    Err(err) => style::display_error(&err),
                                }
                            } else {
                                println!("{}", shim_name.to_string_lossy());
                            }
                            Ok(false)
                        })
                    })
                })
                .collect::<Vec<_>>()
                .iter()
                .any(|ref result| result.as_ref().ok() == Some(&true))
        })
}

fn create(_session: &Session, shim_name: String, _verbose: bool) -> Fallible<bool> {
    path::create_shim_symlink(&shim_name)?;
    Ok(true)
}

fn resolve_shim(session: &mut Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    match shim_name.to_str() {
        Some("node") => resolve_node_shims(session, shim_name),
        Some("npm") => resolve_node_shims(session, shim_name),
        Some("yarn") => resolve_yarn_shims(session, shim_name),
        Some("npx") => resolve_npx_shims(session, shim_name),
        Some(_) => resolve_3p_shims(session, shim_name),
        None => panic!("Cannot format {} as a string", shim_name.to_string_lossy()),
    }
}

fn resolve_node_shims(session: &mut Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    let version = session.current_node()?;
    version.map_or(Ok(ShimKind::System), |v| {
        let mut bin_path = path::node_version_bin_dir(&v.to_string()).unknown()?;
        bin_path.push(&shim_name);
        Ok(ShimKind::Global(bin_path))
    })
}

fn resolve_yarn_shims(session: &mut Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    let version = session.current_yarn()?;
    version.map_or(Ok(ShimKind::System), |v| {
        let mut bin_path = path::yarn_version_bin_dir(&v.to_string()).unknown()?;
        bin_path.push(&shim_name);
        Ok(ShimKind::Global(bin_path))
    })
}

fn resolve_npx_shims(_session: &mut Session, _shim_name: &OsStr) -> Fallible<ShimKind> {
    Ok(ShimKind::Unimplemented)
}

fn resolve_3p_shims(session: &mut Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    // if this is a local executable, get the path to that
    if let Some(project) = session.project() {
        if project.has_local_bin(shim_name)? {
            let mut path_to_bin = project.local_bin_dir();
            path_to_bin.push(shim_name);
            return Ok(ShimKind::Local(path_to_bin));
        }
    }
    // if node is configured with Notion, use the global executable
    // otherwise it's a shim to system executables
    let version = session.current_node()?;
    version.map_or(Ok(ShimKind::System), |v| {
        let mut third_p_bin_dir = path::node_version_3p_bin_dir(&v.to_string())?;
        third_p_bin_dir.push(&shim_name);
        Ok(ShimKind::Global(third_p_bin_dir))
    })
}
