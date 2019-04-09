//! Provides utilities for modifying shims for 3rd-party executables

use std::{fs, io};

use crate::error::ErrorDetails;
use crate::path;
use notion_fail::{throw, FailExt, Fallible};

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
