use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::PathBuf;

use notion_core::session::{ActivityKind, Session};
use notion_core::{path, shim, style};
use notion_fail::{Fallible, ResultExt};
use semver::VersionReq;

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
    Local(PathBuf),
    Global(PathBuf),
    System,
    WillInstall(VersionReq),
    Unimplemented,
}

impl Display for ShimKind {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            &ShimKind::Local(ref path) => format!("{}", path.to_string_lossy()),
            &ShimKind::Global(ref path) => format!("{}", path.to_string_lossy()),
            &ShimKind::System => format!("[system]"),
            &ShimKind::WillInstall(ref version) => format!("[will install version {}]", version),
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

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Shim);

        let result = match self {
            Shim::Help => Help::Command(CommandName::Shim).run(session),
            Shim::List(verbose) => list(session, verbose),
            Shim::Create(shim_name, verbose) => create(session, shim_name, verbose),
            Shim::Delete(shim_name, verbose) => delete(session, shim_name, verbose),
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
                        f.path().file_name().map_or(Ok(false), |shim_name| {
                            if verbose {
                                match resolve_shim(session, &shim_name) {
                                    Ok(shim_info) => {
                                        println!("{} -> {}", shim_name.to_string_lossy(), shim_info)
                                    },
                                    Err(err) => {
                                        style::display_error(&err);
                                        return Ok(false);
                                    },
                                }
                            } else {
                                println!("{}", shim_name.to_string_lossy());
                            }
                            Ok(true)
                        })
                    })
                })
                .collect::<Vec<_>>()
                .iter()
                // return false if anything failed
                .all(|ref result| result.as_ref().ok() == Some(&true))
        })
}

fn create(_session: &Session, shim_name: String, _verbose: bool) -> Fallible<bool> {
    shim::create(&shim_name)?;
    Ok(true)
}

fn delete(_session: &Session, shim_name: String, _verbose: bool) -> Fallible<bool> {
    shim::delete(&shim_name)?;
    Ok(true)
}

fn resolve_shim(session: &mut Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    match shim_name.to_str() {
        Some("node") | Some("npm") => resolve_node_shims(session, shim_name),
        Some("yarn") => resolve_yarn_shims(session, shim_name),
        Some("npx") => resolve_npx_shims(session, shim_name),
        Some(_) => resolve_3p_shims(session, shim_name),
        None => panic!("Cannot format {} as a string", shim_name.to_string_lossy()),
    }
}

// figure out which version of node is installed or configured,
// or which version will be installed if it's not available locally
fn resolve_node_shims(session: &Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    if let Some(project) = session.project() {
        let requirements = &project.manifest().node;
        let catalog = session.catalog()?;
        if let Some(available) = catalog.node.resolve_local(&requirements) {
            // node is available locally - this shim will use that version
            let mut bin_path = path::node_version_bin_dir(&available.to_string()).unknown()?;
            bin_path.push(&shim_name);
            return Ok(ShimKind::Global(bin_path));
        }

        // not installed, but will install based on the required version
        return Ok(ShimKind::WillInstall(requirements.clone()));
    }

    if let Some(global_version) = session.global_node()? {
        let mut bin_path = path::node_version_bin_dir(&global_version.to_string()).unknown()?;
        bin_path.push(&shim_name);
        return Ok(ShimKind::Global(bin_path));
    }
    Ok(ShimKind::System)
}

fn resolve_yarn_shims(session: &Session, shim_name: &OsStr) -> Fallible<ShimKind> {
    if let Some(project) = session.project() {
        if let Some(requirements) = &project.manifest().yarn {
            let catalog = session.catalog()?;
            if let Some(available) = catalog.yarn.resolve_local(&requirements) {
                // yarn is available locally - this shim will use that version
                let mut bin_path = path::yarn_version_bin_dir(&available.to_string()).unknown()?;
                bin_path.push(&shim_name);
                return Ok(ShimKind::Global(bin_path));
            }

            // not installed, but will install based on the required version
            return Ok(ShimKind::WillInstall(requirements.clone()));
        }
    }

    if let Some(ref default_version) = session.catalog()?.yarn.default {
        let mut bin_path = path::yarn_version_bin_dir(&default_version.to_string()).unknown()?;
        bin_path.push(&shim_name);
        return Ok(ShimKind::Global(bin_path));
    }
    Ok(ShimKind::System)
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
