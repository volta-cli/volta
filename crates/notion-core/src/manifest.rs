//! Provides the `Manifest` type, which represents a Node manifest file (`package.json`).

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use notion_fail::{Fallible, ResultExt};
use semver::VersionReq;
use serde_json;

use serial;

/// A Node manifest file.
pub struct Manifest {
    /// The requested version of Node, under the `notion.node` key.
    pub node: VersionReq,
    /// The requested version of Yarn, under the `notion.yarn` key.
    pub yarn: Option<VersionReq>,
    /// The `dependencies` section.
    pub dependencies: HashMap<String, String>,
    /// The command to run a plugin for events, under the `notion.events_plugin` key.
    pub events_plugin: Option<String>,
}

impl Manifest {
    /// Loads and parses a Node manifest for the project rooted at the specified path.
    pub fn for_dir(project_root: &Path) -> Fallible<Option<Manifest>> {
        let file = File::open(project_root.join("package.json")).unknown()?;
        let serial: serial::manifest::Manifest = serde_json::de::from_reader(file).unknown()?;
        serial.into_manifest()
    }
}
