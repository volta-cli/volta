//! Provides utilities for modifying shims for 3rd-party executables

use std::{fs, io};

use crate::error::ErrorDetails;
use crate::path;
use jetson_fail::{throw, FailExt, Fallible};

#[derive(PartialEq)]
pub enum ShimResult {
    Created,
    AlreadyExists,
    Deleted,
    DoesntExist,
}

fn is_3p_shim(name: &str) -> bool {
    match name {
        "node" | "yarn" | "npm" | "npx" => false,
        _ => true,
    }
}

pub fn create(shim_name: &str) -> Fallible<ShimResult> {
    let executable = path::shim_executable()?;
    let shim = path::shim_file(shim_name)?;

    #[cfg(windows)]
    windows::create_git_bash_script(shim_name)?;

    match path::create_file_symlink(executable, shim) {
        Ok(_) => Ok(ShimResult::Created),
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                Ok(ShimResult::AlreadyExists)
            } else {
                throw!(err.with_context(|_| ErrorDetails::ShimCreateError {
                    name: shim_name.to_string(),
                }));
            }
        }
    }
}

pub fn delete(shim_name: &str) -> Fallible<ShimResult> {
    if !is_3p_shim(shim_name) {
        throw!(ErrorDetails::ShimRemoveBuiltInError {
            name: shim_name.to_string(),
        });
    }
    let shim = path::shim_file(shim_name)?;

    #[cfg(windows)]
    windows::delete_git_bash_script(shim_name)?;

    match fs::remove_file(shim) {
        Ok(_) => Ok(ShimResult::Deleted),
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                Ok(ShimResult::DoesntExist)
            } else {
                throw!(err.with_context(|_| ErrorDetails::ShimRemoveError {
                    name: shim_name.to_string(),
                }));
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
    use crate::error::ErrorDetails;
    use crate::path;
    use jetson_fail::{FailExt, Fallible, ResultExt};
    use std::fs::{remove_file, write};
    use std::io::ErrorKind;

    const BASH_SCRIPT: &'static str = r#"cmd //C $0 "$@""#;

    pub fn create_git_bash_script(shim_name: &str) -> Fallible<()> {
        let script_path = path::shim_git_bash_script_file(shim_name)?;
        write(script_path, BASH_SCRIPT).with_context(|_| ErrorDetails::ShimCreateError {
            name: shim_name.to_string(),
        })
    }

    pub fn delete_git_bash_script(shim_name: &str) -> Fallible<()> {
        let script_path = path::shim_git_bash_script_file(shim_name)?;
        remove_file(script_path).or_else(|e| {
            if e.kind() == ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e.with_context(|_| ErrorDetails::ShimRemoveError {
                    name: shim_name.to_string(),
                }))
            }
        })
    }
}
