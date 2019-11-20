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
use volta_core::layout::volta_home;
#[cfg(unix)]
use volta_core::layout::volta_install;
use volta_core::shim;
use volta_fail::{Fallible, ResultExt};

enum MigrationState {
    Empty(empty::Empty),
    V0(Box<v0::V0>),
    V1(Box<v1::V1>),
}

impl MigrationState {
    fn current() -> Fallible<Self> {
        /*
        Triage for determining the current layout version:
        - Does the V1 layout file exist? If yes then V1
        - Does Volta Home exist? If yes (Windows) then V0
            - If yes (Unix) then check if Volta Install is outside shim_dir?
                - If yes, then V0
                - If no, then check if $VOLTA_HOME/load.sh exists? If yes then V0
        - Else Empty

        The extra logic on Unix is necessary because Unix installs can be either inside or outside $VOLTA_HOME
        If it is inside, then the directory necessarily must exist, so we can't use that as a determination.
        If it is outside (and for Windows which is always outside), then if $VOLTA_HOME exists, it must be from a
        previous, V0 installation.

        Going forward, each new version will have an associated layout file, so we can use that as a discriminant
        */

        let home = volta_home()?;
        let volta_home = home.root().to_owned();

        if home.layout_file().exists() {
            return Ok(MigrationState::V1(Box::new(v1::V1::new(volta_home))));
        }

        if volta_home.exists() {
            #[cfg(windows)]
            return Ok(MigrationState::V0(Box::new(v0::V0::new(volta_home))));

            #[cfg(unix)]
            {
                let install = volta_install()?;
                if install.root().starts_with(&volta_home) {
                    // Installed inside $VOLTA_HOME, so need to look for `load.sh` as a marker
                    if volta_home.join("load.sh").exists() {
                        return Ok(MigrationState::V0(Box::new(v0::V0::new(volta_home))));
                    }
                } else {
                    // Installed outside of $VOLTA_HOME, so it must exist from a previous V0 install
                    return Ok(MigrationState::V0(Box::new(v0::V0::new(volta_home))));
                }
            }
        }

        Ok(MigrationState::Empty(empty::Empty::new(volta_home)))
    }
}

pub fn run_migration() -> Fallible<()> {
    let mut state = MigrationState::current()?;

    loop {
        state = match state {
            MigrationState::Empty(e) => MigrationState::V1(Box::new(e.try_into()?)),
            MigrationState::V0(zero) => MigrationState::V1(Box::new((*zero).try_into()?)),
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
    if metadata.file_type().is_symlink() {
        entry
            .path()
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(|stem| stem.to_string())
    } else {
        None
    }
}
