//! Provides the `Manifest` type, which represents a Node manifest file (`package.json`).

use std::collections::{HashMap, HashSet};
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{Context, ErrorKind, Fallible};
use crate::platform::PlatformSpec;
use serde::Serialize;

pub(crate) mod serial;

/// A Node manifest file.
pub struct Manifest {
    /// The platform image specified by the `volta` section.
    pub platform: Option<PlatformSpec>,
    /// The `dependencies` section.
    pub dependencies: HashMap<String, String>,
    /// The `devDependencies` section.
    pub dev_dependencies: HashMap<String, String>,
}

impl Manifest {
    /// Loads and parses a Node manifest for the project rooted at the specified path.
    pub fn for_dir(project_root: &Path) -> Fallible<Manifest> {
        let package_file = project_root.join("package.json");
        let file = File::open(&package_file).with_context(|| ErrorKind::PackageReadError {
            file: package_file.to_path_buf(),
        })?;

        let serial: serial::Manifest =
            serde_json::de::from_reader(file).with_context(|| ErrorKind::PackageParseError {
                file: package_file.to_path_buf(),
            })?;
        serial.into_manifest(&package_file)
    }

    /// Returns a reference to the platform image specified by manifest, if any.
    pub fn platform(&self) -> Option<&PlatformSpec> {
        self.platform.as_ref()
    }

    /// Gets the names of all the direct dependencies in the manifest.
    pub fn merged_dependencies(&self) -> HashSet<String> {
        self.dependencies
            .iter()
            .chain(self.dev_dependencies.iter())
            .map(|(name, _version)| name.clone())
            .collect()
    }

    /// Updates the pinned platform information
    pub fn update_platform(&mut self, platform: PlatformSpec) {
        self.platform = Some(platform);
    }

    /// Updates the `volta` key in the specified `package.json` to match the current Manifest
    pub fn write(&self, package_file: PathBuf) -> Fallible<()> {
        // Helper for lazily creating the file name string without moving `package_file` into
        // one of the individual `with_context` closures below.
        let get_file = || package_file.to_owned();

        // parse the entire package.json file into a Value
        let contents = read_to_string(&package_file)
            .with_context(|| ErrorKind::PackageReadError { file: get_file() })?;

        let is_end_with_newline = contents.ends_with('\n');

        let mut v: serde_json::Value = serde_json::from_str(&contents)
            .with_context(|| ErrorKind::PackageParseError { file: get_file() })?;

        if let Some(map) = v.as_object_mut() {
            // detect indentation in package.json
            let indent = detect_indent::detect_indent(&contents);

            // update the "volta" key
            if let Some(platform) = self.platform() {
                let volta_value = serde_json::to_value(serial::ToolchainSpec::of(platform))
                    .with_context(|| ErrorKind::StringifyToolchainError)?;
                map.insert("volta".to_string(), volta_value);
            } else {
                map.remove("volta");
            }

            // serialize the updated contents back to package.json
            let mut file = File::create(&package_file)
                .with_context(|| ErrorKind::PackageWriteError { file: get_file() })?;
            let formatter =
                serde_json::ser::PrettyFormatter::with_indent(indent.indent().as_bytes());
            let mut ser = serde_json::Serializer::with_formatter(&file, formatter);
            map.serialize(&mut ser)
                .with_context(|| ErrorKind::PackageWriteError { file: get_file() })?;
            // append the empty line if the original package.json has one
            if is_end_with_newline {
                writeln!(file)
                    .with_context(|| ErrorKind::PackageWriteError { file: get_file() })?;
            }
        }
        Ok(())
    }
}

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
