use std::convert::TryFrom;
use std::fs::{remove_file, File};
use std::path::PathBuf;

use super::empty::Empty;
use super::regenerate_shims_for_dir;
use super::v0::V0;
use log::debug;
use volta_core::error::ErrorDetails;
use volta_core::fs::read_dir_eager;
use volta_fail::{Fallible, ResultExt, VoltaError};
use volta_layout::v1;

pub struct V1 {
    pub home: v1::VoltaHome,
}

impl V1 {
    pub fn new(home: PathBuf) -> Self {
        V1 {
            home: v1::VoltaHome::new(home),
        }
    }

    pub fn finalize(&self) -> Fallible<()> {
        regenerate_shims_for_dir(self.home.shim_dir())?;

        debug!("Writing layout marker file");
        File::create(self.home.layout_file()).with_context(|_| {
            ErrorDetails::CreateLayoutFileError {
                file: self.home.layout_file().to_owned(),
            }
        })?;

        Ok(())
    }
}

impl TryFrom<Empty> for V1 {
    type Error = VoltaError;

    fn try_from(old: Empty) -> Fallible<V1> {
        debug!("New Volta installation detected, creating fresh layout");

        let home = v1::VoltaHome::new(old.home);
        home.create()
            .with_context(|_| ErrorDetails::CreateDirError {
                dir: home.root().to_owned(),
            })?;

        Ok(V1 { home })
    }
}

impl TryFrom<V0> for V1 {
    type Error = VoltaError;

    fn try_from(old: V0) -> Fallible<V1> {
        debug!("Existing Volta installation detected, migrating from V0 layout");

        let new_home = v1::VoltaHome::new(old.home.root().to_owned());
        new_home
            .create()
            .with_context(|_| ErrorDetails::CreateDirError {
                dir: new_home.root().to_owned(),
            })?;

        debug!("Removing unnecessary 'load.*' files");
        let root_contents =
            read_dir_eager(new_home.root()).with_context(|_| ErrorDetails::ReadDirError {
                dir: new_home.root().to_owned(),
            })?;
        for (entry, _) in root_contents {
            let path = entry.path();
            if let Some(stem) = path.file_stem() {
                if stem == "load" && path.is_file() {
                    remove_file(&path)
                        .with_context(|_| ErrorDetails::DeleteFileError { file: path })?;
                }
            }
        }

        debug!("Removing old Volta binaries");
        let old_volta_bin = new_home.root().join("volta");
        if old_volta_bin.exists() {
            remove_file(&old_volta_bin).with_context(|_| ErrorDetails::DeleteFileError {
                file: old_volta_bin,
            })?;
        }

        let old_shim_bin = new_home.root().join("shim");
        if old_shim_bin.exists() {
            remove_file(&old_shim_bin)
                .with_context(|_| ErrorDetails::DeleteFileError { file: old_shim_bin })?;
        }

        Ok(V1 { home: new_home })
    }
}
