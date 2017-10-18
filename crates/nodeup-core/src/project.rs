use std::fs::File;
use std::path::Path;
use std::ffi::OsStr;

use serde_json;

use manifest::{Manifest, parse};

fn is_node_root(dir: &Path) -> bool {
    dir.join("package.json").is_file()
}

fn is_node_modules(dir: &Path) -> bool {
   dir.file_name() == Some(OsStr::new("node_modules"))
}

fn is_dependency(dir: &Path) -> bool {
    dir.parent().map_or(false, |parent| is_node_modules(parent))
}

pub fn is_project_root(dir: &Path) -> bool {
    is_node_root(dir) && !is_dependency(dir)
}

pub fn find_project_root(mut dir: &Path) -> Option<&Path> {
    while !is_project_root(dir) {
        dir = match dir.parent() {
            Some(parent) => parent,
            None => { return None; }
        }
    }
    return Some(dir);
}

pub fn find_manifest(dir: &Path) -> ::Result<Option<Manifest>> {
    let root = match find_project_root(dir) {
        Some(root) => root,
        None => { return Ok(None); }
    };

    let file = File::open(root.join("package.json"))?;

    parse(serde_json::de::from_reader(file)?)
}
