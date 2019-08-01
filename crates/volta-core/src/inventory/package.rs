use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::{read, read_dir, DirEntry, File};
use std::io::Read;
use std::path::PathBuf;

use failure::ResultExt;
use log::debug;
use semver::Version;

use volta_fail::{Fallible, VoltaError};

use crate::error::ErrorDetails;
use crate::path;
use crate::tool::{Package, PackageConfig};

// Convenience for access as `package::Collection`
pub use PackageCollection as Collection;

pub struct PackageCollection {
    pub packages: BTreeSet<PackageConfig>,
}

impl PackageCollection {
    // loads an empty PackageCollection
    // ISSUE(#288) Collection only supports versions - for packages we also need names
    pub(crate) fn load() -> Fallible<Self> {
        let package_dir = path::user_package_dir()?;

        let dir_contents = read_dir(&package_dir)
            .with_context(|_| ErrorDetails::ReadInventoryDirError { dir: package_dir })?;

        let file_paths = dir_contents
            // Ignore any items which didn't resolve as `DirEntry` correctly.
            // There is no point trying to do anything with those, and no error
            // we can report to the user in any case. Log the failure in the
            // debug output, though
            .filter_map(|entry| match entry {
                Ok(dir_entry) => Some(dir_entry),
                Err(e) => {
                    debug!("{}", e);
                    None
                }
            })
            // Ignore directory entries.
            // TODO: *recurse* through directories before we get to this point,
            //       so that we correctly handle namespaced packages.
            .filter_map(|dir_entry| {
                let is_file = dir_entry
                    .file_type()
                    .map(|file_type| file_type.is_dir())
                    .unwrap_or(false);

                if is_file {
                    Some(PathBuf::from(dir_entry.file_name()))
                } else {
                    None
                }
            })
            .collect::<Vec<PathBuf>>();

        // Note: this approach fails eagerly -- if *any* of the packages
        //       installed error out on deserialization, we die immediately and
        //       report to the user.
        let mut packages: BTreeSet<PackageConfig> = BTreeSet::new();
        for file_path in file_paths {
            let file =
                File::open(&file_path).with_context(|_| ErrorDetails::ReadPackageConfigError {
                    file: file_path.clone(),
                })?;

            let config = PackageConfig::from_file(&file_path)?;

            packages.insert(config);
        }

        Ok(PackageCollection { packages })
    }

    pub(crate) fn contains(&self, name: &str, version: &Version) -> bool {
        self.packages
            .iter()
            .find(|config| config.name == name && &config.version == version)
            .is_some()
    }
}
