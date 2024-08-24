//! Provides types for modeling the current state of the Volta directory and for migrating between versions
//!
//! A new layout should be represented by its own struct (as in the existing v0 or v1 modules)
//! Migrations between types should be represented by `TryFrom` implementations between the layout types
//! (see v1.rs for examples)
//!
//! NOTE: Since the layout file is written once the migration is complete, all migration implementations
//! need to be aware that they may be partially applied (if something fails in the process) and should be
//! able to re-start gracefully from an interrupted migration

use std::path::Path;

mod empty;
mod v0;
mod v1;
mod v2;
mod v3;
mod v4;

use v0::V0;
use v1::V1;
use v2::V2;
use v3::V3;
use v4::V4;

use log::{debug, info};
use volta_core::error::Fallible;
use volta_core::layout::volta_home;
#[cfg(unix)]
use volta_core::layout::volta_install;
use volta_core::shim::regenerate_shims_for_dir;
use volta_core::sync::VoltaLock;

/// Represents the state of the Volta directory at every point in the migration process
///
/// Migrations should be applied sequentially, migrating from V0 to V1 to ... as needed, cycling
/// through the possible MigrationState values.
enum MigrationState {
    Empty(empty::Empty),
    V0(Box<V0>),
    V1(Box<V1>),
    V2(Box<V2>),
    V3(Box<V3>),
    V4(Box<V4>),
}

/// Macro to simplify the boilerplate associated with detecting a tagged state.
///
/// Should be passed a series of tuples, each of which contains (in this order):
///
/// * The layout version (module name from `volta_layout` crate, e.g. `v1`)
/// * The `MigrationState` variant name (e.g. `V1`)
/// * The migration object itself (e.g. `V1` from the v1 module in _this_ crate)
///
/// The tuples should be in reverse chronological order, so that the newest is first, e.g.:
///
/// detect_tagged!((v3, V3, V3), (v2, V2, V2), (v1, V1, V1));
macro_rules! detect_tagged {
    ($(($layout:ident, $variant:ident, $migration:ident)),*) => {
        impl MigrationState {
            fn detect_tagged_state(home: &::std::path::Path) -> Option<Self> {
                None
                $(
                    .or_else(|| detect::$layout(home))
                )*
            }
        }

        mod detect {
            $(
                pub(super) fn $layout(home: &::std::path::Path) -> Option<super::MigrationState> {
                    let volta_home = volta_layout::$layout::VoltaHome::new(home.to_owned());
                    if volta_home.layout_file().exists() {
                        Some(super::MigrationState::$variant(Box::new(super::$migration::new(home.to_owned()))))
                    } else {
                        None
                    }
                }
            )*
        }
    }
}

detect_tagged!((v4, V4, V4), (v3, V3, V3), (v2, V2, V2), (v1, V1, V1));

impl MigrationState {
    fn current() -> Fallible<Self> {
        // First look for a tagged version (V1+). If that can't be found, then go through the triage
        // for detecting a legacy version

        let home = volta_home()?;

        match MigrationState::detect_tagged_state(home.root()) {
            Some(state) => Ok(state),
            None => MigrationState::detect_legacy_state(home.root()),
        }
    }

    #[allow(clippy::unnecessary_wraps)] // Needs to be Fallible for Unix
    fn detect_legacy_state(home: &Path) -> Fallible<Self> {
        /*
        Triage for determining the legacy layout version:
        - Does Volta Home exist?
            - If yes (Windows) then V0
            - If yes (Unix) then check if Volta Install is outside shim_dir?
                - If yes, then V0
                - If no, then check if $VOLTA_HOME/load.sh exists? If yes then V0
        - Else Empty

        The extra logic on Unix is necessary because Unix installs can be either inside or outside $VOLTA_HOME
        If it is inside, then the directory necessarily must exist, so we can't use that as a determination.
        If it is outside (and for Windows which is always outside), then if $VOLTA_HOME exists, it must be from a
        previous, V0 installation.
        */

        let volta_home = home.to_owned();

        if volta_home.exists() {
            #[cfg(windows)]
            return Ok(MigrationState::V0(Box::new(V0::new(volta_home))));

            #[cfg(unix)]
            {
                let install = volta_install()?;
                if install.root().starts_with(&volta_home) {
                    // Installed inside $VOLTA_HOME, so need to look for `load.sh` as a marker
                    if volta_home.join("load.sh").exists() {
                        return Ok(MigrationState::V0(Box::new(V0::new(volta_home))));
                    }
                } else {
                    // Installed outside of $VOLTA_HOME, so it must exist from a previous V0 install
                    return Ok(MigrationState::V0(Box::new(V0::new(volta_home))));
                }
            }
        }

        Ok(MigrationState::Empty(empty::Empty::new(volta_home)))
    }
}

pub fn run_migration() -> Fallible<()> {
    // Acquire an exclusive lock on the Volta directory, to ensure that no other migrations are running.
    // If this fails, however, we still need to run the migration
    match VoltaLock::acquire() {
        Ok(_lock) => {
            // The lock was acquired, so we can be confident that no other migrations are running
            detect_and_migrate()
        }
        Err(_) => {
            debug!("Unable to acquire lock on Volta directory! Running migration anyway.");
            detect_and_migrate()
        }
    }
}

fn detect_and_migrate() -> Fallible<()> {
    info!("Updating your Volta directory. This may take a few moments...");
    let mut state = MigrationState::current()?;

    // To keep the complexity of writing a new migration from continuously increasing, each new
    // layout version only needs to implement a migration from 2 states: Empty and the previously
    // latest version. We then apply the migrations sequentially here: V0 -> V1 -> ... -> VX
    loop {
        state = match state {
            MigrationState::Empty(e) => MigrationState::V3(Box::new(e.try_into()?)),
            MigrationState::V0(zero) => MigrationState::V1(Box::new((*zero).try_into()?)),
            MigrationState::V1(one) => MigrationState::V2(Box::new((*one).try_into()?)),
            MigrationState::V2(two) => MigrationState::V3(Box::new((*two).try_into()?)),
            MigrationState::V3(three) => MigrationState::V4(Box::new((*three).try_into()?)),
            MigrationState::V4(_) => {
                break;
            }
        };
    }

    regenerate_shims_for_dir(volta_home()?.shim_dir())?;

    Ok(())
}
