//! Provides utilities for modifying shims for 3rd-party executables

use std::{fs, io};

use notion_fail::{ExitCode, FailExt, Fallible, NotionFail};
use path;

#[derive(Fail, Debug)]
#[fail(display = "{}", error)]
pub(crate) struct SymlinkError {
    error: String,
}

impl_notion_fail!(SymlinkError, ExitCode::FileSystemError);

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

fn is_3p_shim(name: &str) -> bool {
    match name {
        "node" | "yarn" | "npm" | "npx" => false,
        _ => true,
    }
}

pub fn create(shim_name: &str) -> Fallible<()> {
    let launchbin = path::launchbin_file()?;
    let shim = path::shim_file(shim_name)?;
    match path::create_file_symlink(launchbin, shim) {
        Ok(_) => Ok(()),
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                throw!(SymlinkError {
                    error: format!("shim `{}` already exists", shim_name),
                });
            } else {
                throw!(err.with_context(SymlinkError::from_io_error));
            }
        }
    }
}

pub fn delete(shim_name: &str) -> Fallible<()> {
    if !is_3p_shim(shim_name) {
        throw!(SymlinkError {
            error: format!("cannot delete `{}`, not a 3rd-party executable", shim_name),
        });
    }
    let shim = path::shim_file(shim_name)?;
    match fs::remove_file(shim) {
        Ok(_) => Ok(()),
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                throw!(SymlinkError {
                    error: format!("shim `{}` does not exist", shim_name),
                });
            } else {
                throw!(err.with_context(SymlinkError::from_io_error));
            }
        }
    }
}
