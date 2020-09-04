use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use crate::error::{Context, ErrorKind, Fallible};
use crate::version::version_serde;
use semver::Version;

#[derive(serde::Deserialize)]
pub struct PackageManifest {
    /// The name of the package
    pub name: String,
    /// The version of the package
    #[serde(deserialize_with = "version_serde::deserialize")]
    pub version: Version,
    /// The `bin` section, containing a map of binary names to locations
    #[serde(default, deserialize_with = "serde_bins::deserialize")]
    pub bin: HashMap<String, String>,
}

impl PackageManifest {
    pub fn for_dir(package: &str, package_root: &Path) -> Fallible<Self> {
        let package_file = package_root.join("package.json");
        let file =
            File::open(&package_file).with_context(|| ErrorKind::PackageManifestReadError {
                package: package.into(),
            })?;

        serde_json::de::from_reader(file).with_context(|| ErrorKind::PackageManifestParseError {
            package: package.into(),
        })
    }
}

mod serde_bins {
    use std::collections::HashMap;
    use std::fmt;

    use serde::de::{Deserializer, Error, MapAccess, Visitor};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(BinMapVisitor)
    }

    struct BinMapVisitor;

    impl<'de> Visitor<'de> for BinMapVisitor {
        type Value = HashMap<String, String>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("string or map")
        }

        // Handle String values with only the path
        fn visit_str<E>(self, bin_path: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            let mut bin_map = HashMap::new();
            // Add the value to the map with the empty string as the key since we don't know
            // what the binary name will be at this level (npm uses the package name in this case)
            bin_map.insert("".into(), bin_path.into());
            Ok(bin_map)
        }

        // Handle maps of Name -> Path
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut bin_map = HashMap::new();
            while let Some((name, path)) = access.next_entry()? {
                bin_map.insert(name, path);
            }
            Ok(bin_map)
        }
    }
}
