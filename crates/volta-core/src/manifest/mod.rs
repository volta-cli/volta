//! Provides the `Manifest` type, which represents a Node manifest file (`package.json`).

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use crate::error::{Context, ErrorKind, Fallible};

pub(crate) mod serial;

pub struct BinManifest {
    /// The `bin` section, containing a map of binary names to locations.
    pub bin: HashMap<String, String>,
    /// The `engines` section, containing a spec of the Node versions that the package works on.
    pub engine: Option<String>,
}

impl BinManifest {
    pub fn for_dir(project_root: &Path) -> Fallible<Self> {
        let package_file = project_root.join("package.json");
        let file = File::open(&package_file).with_context(|| ErrorKind::PackageReadError {
            file: package_file.to_path_buf(),
        })?;

        serde_json::de::from_reader::<File, serial::RawBinManifest>(file)
            .with_context(|| ErrorKind::PackageParseError {
                file: package_file.to_path_buf(),
            })
            .map(BinManifest::from)
    }
}
