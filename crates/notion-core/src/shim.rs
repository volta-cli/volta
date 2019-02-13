//! Provides utilities for modifying shims for 3rd-party executables

use std::{fs, io};

use failure::Fail;

use crate::path;
use notion_fail::{throw, ExitCode, FailExt, Fallible, NotionFail};
use notion_fail_derive::*;

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "{}", error)]
#[notion_fail(code = "FileSystemError")]
pub(crate) struct SymlinkError {
    error: String,
}

impl SymlinkError {
    pub(crate) fn from_io_error(error: &io::Error) -> Self {
        if let Some(inner_err) = error.get_ref() {
            SymlinkError {
                error: inner_err.to_string(),
            }
        } else {
            SymlinkError {
                error: error.to_string(),
            }
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
    let launchtool = path::launchtool_file()?;
    let shim = path::shim_file(shim_name)?;
    match path::create_file_symlink(launchtool, shim) {
        Ok(_) => Ok(ShimResult::Created),
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                Ok(ShimResult::AlreadyExists)
            } else {
                throw!(err.with_context(SymlinkError::from_io_error));
            }
        }
    }
}

pub fn delete(shim_name: &str) -> Fallible<ShimResult> {
    if !is_3p_shim(shim_name) {
        throw!(SymlinkError {
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
                throw!(err.with_context(SymlinkError::from_io_error));
            }
        }
    }
}
