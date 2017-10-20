use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::env;

use manifest::{self, Manifest};
use lockfile::{self, Lockfile};

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
    root: PathBuf,
    manifest: Manifest,
    lockfile: Option<Lockfile>
}

impl Project {
    pub fn for_current_dir() -> ::Result<Option<Project>> {
        let mut dir: &Path = &env::current_dir()?;

        while !is_project_root(dir) {
            dir = match dir.parent() {
                Some(parent) => parent,
                None => { return Ok(None); }
            }
        }

        let manifest = match manifest::read(&dir)? {
            Some(manifest) => manifest,
            None => { return Ok(None); }
        };

        Ok(Some(Project {
            root: dir.to_path_buf(),
            manifest: manifest,
            lockfile: None
        }))
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn lockfile(&mut self) -> ::Result<&Lockfile> {
        self.lockfile = Some(if !lockfile::exists(&self.root) {
            let lockfile = self.manifest.resolve()?;
            lockfile.save(&self.root)?;
            lockfile
        } else {
            let mut lockfile = lockfile::read(&self.root)?;
            if !self.manifest.matches(&lockfile) {
                lockfile = self.manifest.resolve()?;
                lockfile.save(&self.root)?;
            }
            lockfile
        });
        Ok(self.lockfile.as_ref().unwrap())
    }
}
