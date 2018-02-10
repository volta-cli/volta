use std::path::Path;
use std::ffi::OsStr;
use std::env;

use failure;

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

pub struct Project {
    manifest: Manifest
}

impl Project {
    pub fn for_current_dir() -> Result<Option<Project>, failure::Error> {
        let mut dir: &Path = &env::current_dir()?;

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

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}
