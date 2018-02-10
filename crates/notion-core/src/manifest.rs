use std::collections::HashMap;
use std::path::Path;
use std::fs::File;

use serde_json;
use semver::VersionReq;
use failure;

use serial;

pub struct Manifest {
    pub node: VersionReq,
    pub yarn: Option<VersionReq>,
    pub dependencies: HashMap<String, String>
}

// const LATEST_URL: &'static str = "http://nodejs.org/dist/latest/SHASUMS256.txt";

impl Manifest {
    pub fn for_dir(project_root: &Path) -> Result<Option<Manifest>, failure::Error> {
        let file = File::open(project_root.join("package.json"))?;
        let serial: serial::manifest::Manifest = serde_json::de::from_reader(file)?;
        serial.into_manifest()
    }
}
