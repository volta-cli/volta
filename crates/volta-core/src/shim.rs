//! Provides utilities for modifying shims for 3rd-party executables

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;

use crate::error::{Context, ErrorKind, Fallible, VoltaError};
use crate::fs::read_dir_eager;
use crate::layout::volta_home;
use crate::sync::VoltaLock;
use log::debug;

pub use platform::create;

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
        let mut shims: HashSet<String> =
            contents.filter_map(platform::entry_to_shim_name).collect();
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
        Ok(contents.filter_map(platform::entry_to_shim_name).collect())
    }
}

#[derive(PartialEq, Eq)]
pub enum ShimResult {
    Created,
    AlreadyExists,
    Deleted,
    DoesntExist,
}

pub fn delete(shim_name: &str) -> Fallible<ShimResult> {
    let shim = volta_home()?.shim_file(shim_name);

    #[cfg(windows)]
    platform::delete_git_bash_script(shim_name)?;

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

#[cfg(unix)]
mod platform {
    //! Unix-specific shim utilities
    //!
    //! On macOS and Linux, creating a shim involves creating a symlink to the `volta-shim`
    //! executable. Additionally, filtering the shims from directory entries means looking
    //! for symlinks and ignoring the actual binaries
    use std::ffi::OsStr;
    use std::fs::{DirEntry, Metadata};
    use std::io;

    use super::ShimResult;
    use crate::error::{ErrorKind, Fallible, VoltaError};
    use crate::fs::symlink_file;
    use crate::layout::{volta_home, volta_install};

    pub fn create(shim_name: &str) -> Fallible<ShimResult> {
        let executable = volta_install()?.shim_executable();
        let shim = volta_home()?.shim_file(shim_name);

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

    pub fn entry_to_shim_name((entry, metadata): (DirEntry, Metadata)) -> Option<String> {
        if metadata.file_type().is_symlink() {
            entry
                .path()
                .file_stem()
                .and_then(OsStr::to_str)
                .map(ToOwned::to_owned)
        } else {
            None
        }
    }
}

#[cfg(windows)]
mod platform {
    //! Windows-specific shim utilities
    //!
    //! On Windows, creating a shim involves creating a small .cmd script, rather than a symlink.
    //! This allows us to create shims without requiring administrator privileges or developer
    //! mode. Also, to support Git Bash, we create a similar script with bash syntax that doesn't
    //! have a file extension. This allows Powershell and Cmd to ignore it, while Bash detects it
    //! as an executable script.
    //!
    //! Finally, filtering directory entries to find the shim files involves looking for the .cmd
    //! files.
    use std::ffi::OsStr;
    use std::fs::{write, DirEntry, Metadata};

    use super::ShimResult;
    use crate::error::{Context, ErrorKind, Fallible};
    use crate::fs::remove_file_if_exists;
    use crate::layout::volta_home;

    const SHIM_SCRIPT_CONTENTS: &str = r#"@echo off
volta run %~n0 %*
"#;

    const GIT_BASH_SCRIPT_CONTENTS: &str = r#"#!/bin/bash
volta run "$(basename $0)" "$@""#;

    pub fn create(shim_name: &str) -> Fallible<ShimResult> {
        let shim = volta_home()?.shim_file(shim_name);

        write(shim, SHIM_SCRIPT_CONTENTS).with_context(|| ErrorKind::ShimCreateError {
            name: shim_name.to_owned(),
        })?;

        let git_bash_script = volta_home()?.shim_git_bash_script_file(shim_name);

        write(git_bash_script, GIT_BASH_SCRIPT_CONTENTS).with_context(|| {
            ErrorKind::ShimCreateError {
                name: shim_name.to_owned(),
            }
        })?;

        Ok(ShimResult::Created)
    }

    pub fn entry_to_shim_name((entry, _): (DirEntry, Metadata)) -> Option<String> {
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "cmd") {
            path.file_stem()
                .and_then(OsStr::to_str)
                .map(ToOwned::to_owned)
        } else {
            None
        }
    }

    pub fn delete_git_bash_script(shim_name: &str) -> Fallible<()> {
        let script_path = volta_home()?.shim_git_bash_script_file(shim_name);
        remove_file_if_exists(script_path).with_context(|| ErrorKind::ShimRemoveError {
            name: shim_name.to_string(),
        })
    }
}
