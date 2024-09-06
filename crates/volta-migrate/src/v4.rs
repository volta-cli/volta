use std::fs::File;
use std::path::PathBuf;

use super::empty::Empty;
use super::v3::V3;
use log::debug;
use volta_core::error::{Context, ErrorKind, Fallible, VoltaError};
#[cfg(windows)]
use volta_core::fs::read_dir_eager;
use volta_core::fs::remove_file_if_exists;
use volta_layout::v4;

/// Represents a V4 Volta Layout (used by Volta v2.0.0 and above)
///
/// Holds a reference to the V4 layout struct to support potential future migrations
pub struct V4 {
    pub home: v4::VoltaHome,
}

impl V4 {
    pub fn new(home: PathBuf) -> Self {
        V4 {
            home: v4::VoltaHome::new(home),
        }
    }

    /// Write the layout file to mark migration to V4 as complete
    ///
    /// Should only be called once all other migration steps are finished, so that we don't
    /// accidentally mark an incomplete migration as completed
    fn complete_migration(home: v4::VoltaHome) -> Fallible<Self> {
        debug!("Writing layout marker file");
        File::create(home.layout_file()).with_context(|| ErrorKind::CreateLayoutFileError {
            file: home.layout_file().to_owned(),
        })?;

        Ok(V4 { home })
    }
}

impl TryFrom<Empty> for V4 {
    type Error = VoltaError;

    fn try_from(old: Empty) -> Fallible<V4> {
        debug!("New Volta installation detected, creating fresh layout");

        let home = v4::VoltaHome::new(old.home);
        home.create().with_context(|| ErrorKind::CreateDirError {
            dir: home.root().to_owned(),
        })?;

        V4::complete_migration(home)
    }
}

impl TryFrom<V3> for V4 {
    type Error = VoltaError;

    fn try_from(old: V3) -> Fallible<V4> {
        debug!("Migrating from V3 layout");

        let new_home = v4::VoltaHome::new(old.home.root().to_owned());
        new_home
            .create()
            .with_context(|| ErrorKind::CreateDirError {
                dir: new_home.root().to_owned(),
            })?;

        // Perform the core of the migration
        #[cfg(windows)]
        {
            migrate_shims(&new_home)?;
            migrate_shared_directory(&new_home)?;
        }

        // Complete the migration, writing the V4 layout file
        let layout = V4::complete_migration(new_home)?;

        // Remove the V3 layout file, since we're now on V4 (do this after writing the V4 so that we know the migration succeeded)
        let old_layout_file = old.home.layout_file();
        remove_file_if_exists(old_layout_file)?;
        Ok(layout)
    }
}

/// Migrate Windows shims to use the new non-symlink approach. Previously, shims were created in
/// the same way as on Unix: With symlinks to the `volta-shim` executable. Now, we use scripts that
/// call `volta run` to execute the underlying tool. This allows us to avoid needing developer
/// mode, making Volta more broadly usable for Windows devs.
///
/// To migrate the shims, we read the shim directory looking for symlinks, remove those, and then
/// file stem (name without extension) to generate new shims.
#[cfg(windows)]
fn migrate_shims(new_home: &v4::VoltaHome) -> Fallible<()> {
    use std::ffi::OsStr;

    let entries = read_dir_eager(new_home.shim_dir()).with_context(|| ErrorKind::ReadDirError {
        dir: new_home.shim_dir().to_owned(),
    })?;

    for (entry, metadata) in entries {
        if metadata.is_symlink() {
            let path = entry.path();
            remove_file_if_exists(&path)?;

            if let Some(shim_name) = path.file_stem().and_then(OsStr::to_str) {
                volta_core::shim::create(shim_name)?;
            }
        }
    }

    Ok(())
}

/// Migrate Windows shared directory to use junctions rather than directory symlinks. Similar to
/// the shims, we previously used symlinks to create the shared global package directory, which
/// requires developer mode. By using junctions, we can avoid that requirement entirely.
///
/// To migrate the directories, we read the shim directory, determine the target of each symlink,
/// delete the link, and then create a junction (using volta_core::fs::symlink_dir which delegates
/// to `junction` internally)
#[cfg(windows)]
fn migrate_shared_directory(new_home: &v4::VoltaHome) -> Fallible<()> {
    use std::fs::read_link;
    use volta_core::fs::{remove_dir_if_exists, symlink_dir};

    let entries =
        read_dir_eager(new_home.shared_lib_root()).with_context(|| ErrorKind::ReadDirError {
            dir: new_home.shared_lib_root().to_owned(),
        })?;

    for (entry, metadata) in entries {
        if metadata.is_symlink() {
            let path = entry.path();
            let source = read_link(&path).with_context(|| ErrorKind::ReadDirError {
                dir: new_home.shared_lib_root().to_owned(),
            })?;

            remove_dir_if_exists(&path)?;
            symlink_dir(source, path).with_context(|| ErrorKind::CreateSharedLinkError {
                name: entry.file_name().to_string_lossy().to_string(),
            })?;
        }
    }

    Ok(())
}
