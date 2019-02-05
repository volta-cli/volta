//! Provides utilities for modifying shims for 3rd-party executables

use std::{fs, io};

use crate::error::ErrorDetails;
use crate::path;
use notion_fail::{throw, FailExt, Fallible};

fn symlink_error(error: &io::Error) -> ErrorDetails {
    if let Some(inner_err) = error.get_ref() {
        ErrorDetails::SymlinkError {
            error: inner_err.to_string(),
        }
    } else {
        ErrorDetails::SymlinkError {
            error: error.to_string(),
        }
    }
}

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
    // TODO: this should be a hard link?
    match path::create_file_symlink(executable, shim) {
        Ok(_) => Ok(ShimResult::Created),
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                Ok(ShimResult::AlreadyExists)
            } else {
                throw!(err.with_context(symlink_error));
            }
        }
    }
}

pub fn delete(shim_name: &str) -> Fallible<ShimResult> {
    if !is_3p_shim(shim_name) {
        throw!(ErrorDetails::SymlinkError {
            error: format!("cannot delete `{}`, not a 3rd-party executable", shim_name),
        });
    }
    let shim = path::shim_file(shim_name)?;
    match fs::remove_file(shim) {
        Ok(_) => Ok(ShimResult::Deleted),
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                Ok(ShimResult::DoesntExist)
            } else {
                throw!(err.with_context(symlink_error));
            }
        }
    }
}
