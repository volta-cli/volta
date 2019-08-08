use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs::{read, read_dir, DirEntry, File},
    io::Read,
    iter::Iterator,
    path::PathBuf,
};

use failure::ResultExt;
use log::debug;
use semver::Version;
use walkdir::WalkDir;

use crate::error::ErrorDetails;
use crate::path;
use crate::tool::{Package, PackageConfig};
use volta_fail::{Fallible, VoltaError};

// Convenience for access as `package::Collection`
pub use PackageCollection as Collection;

#[derive(Clone)]
pub struct PackageCollection(BTreeSet<PackageConfig>);

impl PackageCollection {
    pub(crate) fn load() -> Fallible<Self> {
        let package_dir = path::user_package_dir()?;

        let file_paths = WalkDir::new(&package_dir)
            .max_depth(1)
            .into_iter()
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
            .filter_map(|dir_entry| {
                if dir_entry.file_type().is_dir() {
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

        Ok(PackageCollection(packages))
    }

    pub(crate) fn contains(&self, name: &str, version: &Version) -> bool {
        self.0
            .iter()
            .find(|config| config.name == name && &config.version == version)
            .is_some()
    }
}

impl IntoIterator for PackageCollection {
    type Item = PackageConfig;
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
