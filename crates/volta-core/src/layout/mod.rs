use std::env;
use std::path::PathBuf;

use crate::error::{Context, ErrorKind, Fallible};
use cfg_if::cfg_if;
use dunce::canonicalize;
use once_cell::sync::OnceCell;
use volta_layout::v4::{VoltaHome, VoltaInstall};

cfg_if! {
    if #[cfg(unix)] {
        mod unix;
        pub use unix::*;
    } else if #[cfg(windows)] {
        mod windows;
        pub use windows::*;
    }
}

static VOLTA_HOME: OnceCell<VoltaHome> = OnceCell::new();
static VOLTA_INSTALL: OnceCell<VoltaInstall> = OnceCell::new();

pub fn volta_home<'a>() -> Fallible<&'a VoltaHome> {
    VOLTA_HOME.get_or_try_init(|| {
        let home_dir = match env::var_os("VOLTA_HOME") {
            Some(home) => PathBuf::from(home),
            None => default_home_dir()?,
        };

        Ok(VoltaHome::new(home_dir))
    })
}

pub fn volta_install<'a>() -> Fallible<&'a VoltaInstall> {
    VOLTA_INSTALL.get_or_try_init(|| {
        let install_dir = match env::var_os("VOLTA_INSTALL_DIR") {
            Some(install) => PathBuf::from(install),
            None => default_install_dir()?,
        };

        Ok(VoltaInstall::new(install_dir))
    })
}

/// Determine the binary install directory from the currently running executable
///
/// The volta-shim and volta binaries will be installed in the same location, so we can use the
/// currently running executable to find the binary install directory. Note that we need to
/// canonicalize the path we get from current_exe to make sure we resolve symlinks and find the
/// actual binary files
fn default_install_dir() -> Fallible<PathBuf> {
    env::current_exe()
        .and_then(canonicalize)
        .map(|mut path| {
            path.pop(); // Remove the executable name from the path
            path
        })
        .with_context(|| ErrorKind::NoInstallDir)
}
