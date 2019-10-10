use std::env;
use std::path::PathBuf;

use crate::error::ErrorDetails;
#[cfg(unix)]
use crate::shim;
use cfg_if::cfg_if;
use double_checked_cell::DoubleCheckedCell;
use lazy_static::lazy_static;
use volta_fail::{Fallible, ResultExt};
use volta_layout::v0::{VoltaHome, VoltaInstall};

cfg_if! {
    if #[cfg(unix)] {
        mod unix;
        pub use unix::*;
    } else if #[cfg(windows)] {
        mod windows;
        pub use windows::*;
    }
}

lazy_static! {
    static ref VOLTA_HOME: DoubleCheckedCell<VoltaHome> = DoubleCheckedCell::new();
    static ref VOLTA_INSTALL: DoubleCheckedCell<VoltaInstall> = DoubleCheckedCell::new();
}

pub fn volta_home<'a>() -> Fallible<&'a VoltaHome> {
    VOLTA_HOME.get_or_try_init(|| {
        let home_dir = match env::var_os("VOLTA_HOME") {
            Some(home) => PathBuf::from(home),
            None => default_home_dir()?,
        };

        Ok(VoltaHome::new(home_dir))
    })
}

// NOTE: This initialization will, on some code paths, call volta_home()
// We need to make sure that volta_home does not in turn call this method
// or we will run into problems with deadlocks
pub fn volta_install<'a>() -> Fallible<&'a VoltaInstall> {
    VOLTA_INSTALL.get_or_try_init(|| {
        let install_dir = match env::var_os("VOLTA_INSTALL_DIR") {
            Some(install) => PathBuf::from(install),
            None => default_install_dir()?,
        };

        Ok(VoltaInstall::new(install_dir))
    })
}

pub fn ensure_volta_dirs_exist() -> Fallible<()> {
    let home = volta_home()?;
    if !home.root().exists() {
        home.create()
            .with_context(|_| ErrorDetails::CreateDirError {
                dir: home.root().to_owned(),
            })?;

        // also ensure the basic shims exist
        // this is only for unix until the update process is refactored
        #[cfg(unix)]
        {
            shim::create("node")?;
            shim::create("yarn")?;
            shim::create("npm")?;
            shim::create("npx")?;
        }
    }

    Ok(())
}
