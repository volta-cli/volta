use std::collections::HashMap;
use std::path::Path;
use std::fs::File;

use serde_json;
use semver::VersionReq;
use failure;

use version::VersionSpec;
use serial;

pub struct Manifest {
    pub node: VersionSpec,
    pub yarn: Option<VersionSpec>,
    pub dependencies: HashMap<String, String>
}

// const LATEST_URL: &'static str = "http://nodejs.org/dist/latest/SHASUMS256.txt";

impl Manifest {
    // FIXME: change to return &VersionReq after we stop using the version crate
    pub fn node_req(&self) -> VersionReq {
        match self.node {
            VersionSpec::Specific(ref version) => {
                let version = &version[..];
                let src = version.trim();
                if src.len() > 0 && src.chars().next().unwrap().is_digit(10) {
                    let defaulted = format!("={}", src);
                    VersionReq::parse(&defaulted).unwrap()
                } else {
                    VersionReq::parse(src).unwrap()
                }
            }
            _ => { unimplemented!() }
        }
    }
}

pub fn read(project_root: &Path) -> Result<Option<Manifest>, failure::Error> {
    let file = File::open(project_root.join("package.json"))?;
    let serial: serial::manifest::Manifest = serde_json::de::from_reader(file)?;
    serial.into_manifest()
}
