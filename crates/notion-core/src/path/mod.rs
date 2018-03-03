//! Provides functions for determining the paths in the filesystem for
//! directories and files required in a Notion installation.

#[cfg(not(windows))]
mod unix;

#[cfg(not(windows))]
pub use self::unix::*;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use self::windows::*;

#[derive(Fail, Debug)]
#[fail(display = "Unknown system folder: '{}'", name)]
pub(crate) struct UnknownSystemFolderError {
    name: &'static str
}

pub fn archive_file(version: &str) -> String {
    format!("{}.{}", archive_root_dir(version), archive_extension())
}

pub fn archive_root_dir(version: &str) -> String {
    format!("node-v{}-{}-{}", version, OS, ARCH)
}
