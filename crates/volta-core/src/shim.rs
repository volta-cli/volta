//! Provides utilities for modifying shims for 3rd-party executables

use std::collections::HashSet;
use std::fs::{self, DirEntry, Metadata};
use std::io;
use std::path::Path;

use crate::error::{Context, ErrorKind, Fallible, VoltaError};
use crate::fs::{read_dir_eager, symlink_file};
use crate::layout::{volta_home, volta_install};
use crate::sync::VoltaLock;
use log::debug;

pub fn regenerate_shims_for_dir(dir: &Path) -> Fallible<()> {
    // Acquire a lock on the Volta directory, if possible, to prevent concurrent changes
    let _lock = VoltaLock::acquire();
    debug!("Rebuilding shims for directory: {}", dir.display());
    for shim_name in get_shim_list_deduped(dir)?.iter() {
        delete(shim_name)?;
        create(shim_name)?;
    }

    Ok(())
}

fn get_shim_list_deduped(dir: &Path) -> Fallible<HashSet<String>> {
    let contents = read_dir_eager(dir).with_context(|| ErrorKind::ReadDirError {
        dir: dir.to_owned(),
    })?;

    #[cfg(unix)]
    {
        let mut shims: HashSet<String> = contents.filter_map(entry_to_shim_name).collect();
        shims.insert("node".into());
        shims.insert("npm".into());
        shims.insert("npx".into());
        shims.insert("pnpm".into());
        shims.insert("yarn".into());
        shims.insert("yarnpkg".into());
        Ok(shims)
    }

    #[cfg(windows)]
    {
        // On Windows, the default shims are installed in Program Files, so we don't need to generate them here
        Ok(contents.filter_map(entry_to_shim_name).collect())
    }
}

fn entry_to_shim_name((entry, metadata): (DirEntry, Metadata)) -> Option<String> {
    if metadata.file_type().is_symlink() {
        entry
            .path()
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(|stem| stem.to_string())
    } else {
        None
    }
}

#[derive(PartialEq, Eq)]
pub enum ShimResult {
    Created,
    AlreadyExists,
    Deleted,
    DoesntExist,
}

pub fn create(shim_name: &str) -> Fallible<ShimResult> {
    let executable = volta_install()?.shim_executable();
    let shim = volta_home()?.shim_file(shim_name);

    #[cfg(windows)]
    windows::create_git_bash_script(shim_name)?;

    match symlink_file(executable, shim) {
        Ok(_) => Ok(ShimResult::Created),
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                Ok(ShimResult::AlreadyExists)
            } else {
                Err(VoltaError::from_source(
                    err,
                    ErrorKind::ShimCreateError {
                        name: shim_name.to_string(),
                    },
                ))
            }
        }
    }
}

pub fn delete(shim_name: &str) -> Fallible<ShimResult> {
    let shim = volta_home()?.shim_file(shim_name);

    #[cfg(windows)]
    windows::delete_git_bash_script(shim_name)?;

    match fs::remove_file(shim) {
        Ok(_) => Ok(ShimResult::Deleted),
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                Ok(ShimResult::DoesntExist)
            } else {
                Err(VoltaError::from_source(
                    err,
                    ErrorKind::ShimRemoveError {
                        name: shim_name.to_string(),
                    },
                ))
            }
        }
    }
}

/// These methods are a (hacky) workaround for an issue with Git Bash on Windows
/// When executing the shim symlink, Git Bash resolves the symlink first and then calls shim.exe directly
/// This results in the shim being unable to determine which tool is being executed
/// However, both cmd.exe and PowerShell execute the symlink correctly
/// To fix the issue specifically in Git Bash, we write a bash script in the shim dir, with the same name as the shim
/// minus the '.exe' (e.g. we write `ember` next to the symlink `ember.exe`)
/// Since the file doesn't have a file extension, it is ignored by cmd.exe and PowerShell, but is detected by Bash
/// This bash script simply calls the shim using `cmd.exe`, so that it is resolved correctly
#[cfg(windows)]
mod windows {
    use crate::error::{Context, ErrorKind, Fallible};
    use crate::fs::remove_file_if_exists;
    use crate::layout::volta_home;
    use std::fs::write;

    const BASH_SCRIPT: &str = r#"cmd //C $0 "$@""#;

    pub fn create_git_bash_script(shim_name: &str) -> Fallible<()> {
        let script_path = volta_home()?.shim_git_bash_script_file(shim_name);
        write(script_path, BASH_SCRIPT).with_context(|| ErrorKind::ShimCreateError {
            name: shim_name.to_string(),
        })
    }

    pub fn delete_git_bash_script(shim_name: &str) -> Fallible<()> {
        let script_path = volta_home()?.shim_git_bash_script_file(shim_name);
        remove_file_if_exists(script_path).with_context(|| ErrorKind::ShimRemoveError {
            name: shim_name.to_string(),
        })
    }
}
