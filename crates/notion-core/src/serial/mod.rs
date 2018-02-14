//! Provides utilities for serializing and deserializing file formats.

pub mod catalog;
pub mod manifest;
pub mod config;
pub mod plugin;
pub mod index;
pub mod version;

use std::path::Path;
use std::io;
use std::fs::{File, create_dir_all};

pub fn touch(path: &Path) -> io::Result<File> {
    if !path.is_file() {
        let basedir = path.parent().unwrap();
        create_dir_all(basedir)?;
        File::create(path)?;
    }
    File::open(path)
}
