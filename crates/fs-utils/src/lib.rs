//! This crate provides utilities for operating on the filesystem.

use std::fs;
use std::io;
use std::path::Path;

/// This creates the parent directory of the input path, assuming the input path is a file.
pub fn ensure_containing_dir_exists<P: AsRef<Path>>(path: &P) -> io::Result<()> {
    path.as_ref()
        .parent()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Could not determine directory information for {}",
                    path.as_ref().display()
                ),
            )
        })
        .and_then(fs::create_dir_all)
}
