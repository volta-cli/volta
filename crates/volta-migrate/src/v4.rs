use std::convert::TryFrom;
use std::fs::File;
use std::path::PathBuf;

use crate::empty::Empty;
use crate::v3::V3;
use log::debug;
use volta_core::error::{Context, ErrorKind, Fallible, VoltaError};
use volta_core::fs::remove_file_if_exists;
use volta_layout::v4;

/// Represents a V3 Volta layout (used by Volta v0.9.0 and above)
///
/// Holds a reference to the V3 layout struct to support future migrations
pub struct V4 {
    pub home: v4::VoltaHome,
}

impl V4 {
    pub fn new(home: PathBuf) -> Self {
        V4 {
            home: v4::VoltaHome::new(home),
        }
    }

    /// Write the layout file to mark migration to V2 as complete
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

    fn try_from(old: Empty) -> Fallible<Self> {
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

    fn try_from(old: V3) -> Fallible<Self> {
        debug!("Migrating from V3 layout");

        let new_home = v4::VoltaHome::new(old.home.root().to_owned());
        new_home
            .create()
            .with_context(|| ErrorKind::CreateDirError {
                dir: new_home.root().to_owned(),
            })?;

        // Complete the migration, writing the V3 layout file
        let layout = V4::complete_migration(new_home)?;

        // Remove the V2 layout file, since we're now on V3 (do this after writing the V3 file so that we know the migration succeeded)
        remove_file_if_exists(old.home.layout_file())?;

        Ok(layout)
    }
}
