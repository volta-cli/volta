//! Provides the `PackageInfo` type, which contains data for a Node project (from `package.json`).

use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;

use notion_fail::{Fallible, NotionFail, ResultExt};
use serde_json;

use serial;

#[derive(Fail, Debug)]
#[fail(display = "Could not read package info: {}", error)]
pub(crate) struct PackageReadError {
    pub(crate) error: String,
}

impl PackageReadError {
    pub(crate) fn from_io_error(error: &io::Error) -> Self {
        PackageReadError {
            error: error.to_string(),
        }
    }
}

impl NotionFail for PackageReadError {
    fn is_user_friendly(&self) -> bool {
        true
    }
    fn exit_code(&self) -> i32 {
        4
    }
}

/// Info about a Node package
pub struct PackageInfo {
    /// The `bin` section, containing a map of binary names to locations
    pub bin: HashMap<String, String>,
}

impl PackageInfo {
    /// Loads and parses package.json for the project located at the specified path.
    pub fn for_dir(project_dir: &Path) -> Fallible<PackageInfo> {
        let file = File::open(project_dir.join("package.json"))
            .with_context(PackageReadError::from_io_error)?;
        let serial: serial::package::Info = serde_json::de::from_reader(file).unknown()?;
        Ok(serial.into_package_info())
    }
}

// unit tests

#[cfg(test)]
pub mod tests {

    use package_info::PackageInfo;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn fixture_path(fixture_dir: &str) -> PathBuf {
        let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_manifest_dir.push("fixtures");
        cargo_manifest_dir.push(fixture_dir);
        cargo_manifest_dir
    }

    #[test]
    fn gets_bin_map_format() {
        let project_path = fixture_path("basic/node_modules/eslint");
        let bin = match PackageInfo::for_dir(&project_path) {
            Ok(pkg_info) => pkg_info.bin,
            Err(e) => panic!(
                "Error: Could not get package info for project {:?}, error: {}",
                project_path, e
            ),
        };
        let mut expected_bin = HashMap::new();
        expected_bin.insert("eslint".to_string(), "./bin/eslint.js".to_string());
        assert_eq!(bin, expected_bin);
    }

    #[test]
    fn gets_multiple_bins() {
        let project_path = fixture_path("basic/node_modules/typescript");
        let bin = match PackageInfo::for_dir(&project_path) {
            Ok(pkg_info) => pkg_info.bin,
            Err(e) => panic!(
                "Error: Could not get package info for project {:?}, error: {}",
                project_path, e
            ),
        };
        let mut expected_bin = HashMap::new();
        expected_bin.insert("tsc".to_string(), "./bin/tsc".to_string());
        expected_bin.insert("tsserver".to_string(), "./bin/tsserver".to_string());
        assert_eq!(bin, expected_bin);
    }

    #[test]
    fn gets_bin_string_format() {
        let project_path = fixture_path("basic/node_modules/rsvp");
        let bin = match PackageInfo::for_dir(&project_path) {
            Ok(pkg_info) => pkg_info.bin,
            Err(e) => panic!(
                "Error: Could not get package info for project {:?}, error: {}",
                project_path, e
            ),
        };
        let mut expected_bin = HashMap::new();
        expected_bin.insert("rsvp".to_string(), "./bin/rsvp.js".to_string());
        assert_eq!(bin, expected_bin);
    }

    #[test]
    fn handles_dep_with_no_bin() {
        let project_path = fixture_path("basic/node_modules/@namespace/some-dep");
        let bin = match PackageInfo::for_dir(&project_path) {
            Ok(pkg_info) => pkg_info.bin,
            Err(e) => panic!(
                "Error: Could not get package info for project {:?}, error: {}",
                project_path, e
            ),
        };
        let expected_bin = HashMap::new();
        assert_eq!(bin, expected_bin);
    }
}
