//! Provides utilities for serializing and deserializing file formats.

pub mod catalog;
pub mod manifest;
pub mod config;
pub mod plugin;
pub mod index;
pub mod version;

use std::path::Path;
use std::fs::{File, create_dir_all};

use notion_fail::{Fallible, ResultExt};

pub fn touch(path: &Path) -> Fallible<File> {
    if !path.is_file() {
        let basedir = path.parent().unwrap();
        create_dir_all(basedir).unknown()?;
        File::create(path).unknown()?;
    }
    File::open(path).unknown()
}
