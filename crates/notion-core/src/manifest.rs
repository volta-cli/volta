//! Provides the `Manifest` type, which represents a Node manifest file (`package.json`).

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use notion_fail::{Fallible, ResultExt};
use semver::VersionReq;
use serde_json;

use serial;

/// A Node manifest file.
pub struct Manifest {
    /// The requested version of Node, under the `toolchain.node` key.
    pub node: VersionReq,
    /// The pinned version of Node as a string.
    pub node_str: String,
    /// The requested version of Yarn, under the `toolchain.yarn` key.
    pub yarn: Option<VersionReq>,
    /// The pinned version of Yarn as a string.
    pub yarn_str: Option<String>,
    /// The `dependencies` section.
    pub dependencies: HashMap<String, String>,
    /// The `devDependencies` section.
    pub dev_dependencies: HashMap<String, String>,
}

impl Manifest {
    /// Loads and parses a Node manifest for the project rooted at the specified path.
    pub fn for_dir(project_root: &Path) -> Fallible<Option<Manifest>> {
        let file = File::open(project_root.join("package.json")).unknown()?;
        let serial: serial::manifest::Manifest = serde_json::de::from_reader(file).unknown()?;
        serial.into_manifest()
    }

    pub fn update_toolchain(toolchain: serial::manifest::ToolchainManifest, package_file: PathBuf) -> Fallible<()> {
        // parse the entire package.json file into a Value
        let file = File::open(&package_file).unknown()?;
        let mut v: serde_json::Value = serde_json::from_reader(file).unknown()?;
        if let Some(map) = v.as_object_mut() {
            // update the "toolchain" key
            let toolchain_value = serde_json::to_value(toolchain).unknown()?;
            map.insert("toolchain".to_string(), toolchain_value);
            // write to file
            let file = File::create(package_file).unknown()?;
            serde_json::to_writer_pretty(file, map).unknown()?;
            // TODO: detect indentation and use that
            // (see https://play.rust-lang.org/?gist=009ef8f29aa6af44c26d32be5f3c9724)
        }
        Ok(())
    }
}

// unit tests

#[cfg(test)]
pub mod tests {

    use manifest::Manifest;
    use semver::VersionReq;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn fixture_path(fixture_dir: &str) -> PathBuf {
        let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_manifest_dir.push("fixtures");
        cargo_manifest_dir.push(fixture_dir);
        cargo_manifest_dir
    }

    #[test]
    fn gets_node_version() {
        let project_path = fixture_path("basic");
        let version = match Manifest::for_dir(&project_path) {
            Ok(manifest) => manifest.unwrap().node,
            _ => panic!(
                "Error: Could not get manifest for project {:?}",
                project_path
            ),
        };
        assert_eq!(version, VersionReq::parse("=6.11.1").unwrap());
    }

    #[test]
    fn gets_yarn_version() {
        let project_path = fixture_path("basic");
        let version = match Manifest::for_dir(&project_path) {
            Ok(manifest) => manifest.unwrap().yarn,
            _ => panic!(
                "Error: Could not get manifest for project {:?}",
                project_path
            ),
        };
        assert_eq!(version.unwrap(), VersionReq::parse("=1.2").unwrap());
    }

    #[test]
    fn gets_dependencies() {
        let project_path = fixture_path("basic");
        let dependencies = match Manifest::for_dir(&project_path) {
            Ok(manifest) => manifest.unwrap().dependencies,
            _ => panic!(
                "Error: Could not get manifest for project {:?}",
                project_path
            ),
        };
        let mut expected_deps = HashMap::new();
        expected_deps.insert("@namespace/some-dep".to_string(), "0.2.4".to_string());
        expected_deps.insert("rsvp".to_string(), "^3.5.0".to_string());
        assert_eq!(dependencies, expected_deps);
    }

    #[test]
    fn gets_dev_dependencies() {
        let project_path = fixture_path("basic");
        let dev_dependencies = match Manifest::for_dir(&project_path) {
            Ok(manifest) => manifest.unwrap().dev_dependencies,
            _ => panic!(
                "Error: Could not get manifest for project {:?}",
                project_path
            ),
        };
        let mut expected_deps = HashMap::new();
        expected_deps.insert(
            "@namespaced/something-else".to_string(),
            "^6.3.7".to_string(),
        );
        expected_deps.insert("eslint".to_string(), "~4.8.0".to_string());
        assert_eq!(dev_dependencies, expected_deps);
    }
}
