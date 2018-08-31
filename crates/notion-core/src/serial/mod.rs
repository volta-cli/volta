//! Provides utilities for serializing and deserializing file formats.

pub mod plugin;
pub mod version;

use std::fs::{create_dir_all, File};
use std::path::Path;

use notion_fail::{Fallible, ResultExt};

pub fn touch(path: &Path) -> Fallible<File> {
    if !path.is_file() {
        let basedir = path.parent().unwrap();
        create_dir_all(basedir).unknown()?;
        File::create(path).unknown()?;
    }
    File::open(path).unknown()
}
