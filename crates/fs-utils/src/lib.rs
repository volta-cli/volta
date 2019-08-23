//! This crate provides utilities for operating on the filesystem.

use std::fs;
use std::path::Path;

use failure::{self, Fail};

/// Thrown when the containing directory could not be determined
#[derive(Fail, Debug)]
#[fail(display = "Could not determine directory information for {}", path)]
struct ContainingDirError {
    path: String,
}

/// Thrown when the containing directory could not be determined
#[derive(Fail, Debug)]
#[fail(display = "Could not create directory {}", dir)]
struct CreateDirError {
    dir: String,
}

/// This creates the parent directory of the input path, assuming the input path is a file.
pub fn ensure_containing_dir_exists<P: AsRef<Path>>(path: &P) -> Result<(), failure::Error> {
    path.as_ref()
        .parent()
        .ok_or(
            ContainingDirError {
                path: path.as_ref().to_string_lossy().to_string(),
            }
            .into(),
        )
        .and_then(|dir| {
            fs::create_dir_all(dir).map_err(|_| {
                CreateDirError {
                    dir: dir.to_string_lossy().to_string(),
                }
                .into()
            })
        })
}
