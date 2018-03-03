//! Provides the `Project` type, which represents a Node project tree in
//! the filesystem.

use std::path::Path;
use std::ffi::OsStr;
use std::env;

use error::{Fallible, ResultExt};

use manifest::Manifest;

fn is_node_root(dir: &Path) -> bool {
    dir.join("package.json").is_file()
}

fn is_node_modules(dir: &Path) -> bool {
   dir.file_name() == Some(OsStr::new("node_modules"))
}

fn is_dependency(dir: &Path) -> bool {
    dir.parent().map_or(false, |parent| is_node_modules(parent))
}

fn is_project_root(dir: &Path) -> bool {
    is_node_root(dir) && !is_dependency(dir)
}

/// A Node project tree in the filesystem.
pub struct Project {
    manifest: Manifest
}

impl Project {

    /// Returns the Node project containing the current working directory,
    /// if any.
    pub fn for_current_dir() -> Fallible<Option<Project>> {
        let mut dir: &Path = &env::current_dir().unknown()?;

        while !is_project_root(dir) {
            dir = match dir.parent() {
                Some(parent) => parent,
                None => { return Ok(None); }
            }
        }

        let manifest = match Manifest::for_dir(&dir)? {
            Some(manifest) => manifest,
            None => { return Ok(None); }
        };

        Ok(Some(Project {
            manifest: manifest
        }))
    }

    /// Returns the project manifest (`package.json`) for this project.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}
