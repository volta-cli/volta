#![cfg(feature = "volta-updates")]
use std::collections::HashSet;
use std::convert::TryInto;
use std::fs::{DirEntry, Metadata};
use std::path::Path;

mod empty;
mod v0;
mod v1;

use log::debug;
use volta_core::error::ErrorDetails;
use volta_core::fs::read_dir_eager;
use volta_core::shim;
use volta_fail::{Fallible, ResultExt};

enum MigrationState {
    Empty(empty::Empty),
    V0(v0::V0),
    V1(v1::V1),
}

impl MigrationState {
    fn current() -> Fallible<Self> {
        unimplemented!();
    }
}

pub fn run_migration() -> Fallible<()> {
    let mut state = MigrationState::current()?;

    loop {
        state = match state {
            MigrationState::Empty(e) => MigrationState::V1(e.try_into()?),
            MigrationState::V0(zero) => MigrationState::V1(zero.try_into()?),
            MigrationState::V1(one) => {
                one.finalize()?;
                break;
            }
        };
    }

    Ok(())
}

fn regenerate_shims_for_dir(dir: &Path) -> Fallible<()> {
    debug!("Rebuilding shims");
    for shim_name in get_shim_list_deduped(dir)?.iter() {
        shim::delete(shim_name)?;
        shim::create(shim_name)?;
    }

    Ok(())
}

fn get_shim_list_deduped(dir: &Path) -> Fallible<HashSet<String>> {
    let contents = read_dir_eager(dir).with_context(|_| ErrorDetails::ReadDirError {
        dir: dir.to_owned(),
    })?;

    #[cfg(unix)]
    {
        let mut shims: HashSet<String> = contents.filter_map(entry_to_shim_name).collect();
        shims.insert("node".into());
        shims.insert("npm".into());
        shims.insert("npx".into());
        shims.insert("yarn".into());
        Ok(shims)
    }

    #[cfg(windows)]
    {
        // On Windows, the default shims are installed in Program Files, so we don't need to generate them here
        Ok(contents.filter_map(entry_to_shim_name).collect())
    }
}

fn entry_to_shim_name((entry, metadata): (DirEntry, Metadata)) -> Option<String> {
    if !metadata.is_dir() {
        entry
            .path()
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(|stem| stem.to_string())
    } else {
        None
    }
}
