//! Provides functions for determining the paths of files and directories
//! in a standard Notion layout.

#[cfg(any(not(windows), feature = "universal-docs"))]
#[cfg_attr(feature = "universal-docs", doc(cfg(not(windows))))]
mod unix;

#[cfg(not(windows))]
pub use self::unix::*;

#[cfg(any(windows, feature = "universal-docs"))]
#[cfg_attr(feature = "universal-docs", doc(cfg(windows)))]
mod windows;

#[cfg(windows)]
pub use self::windows::*;

pub fn archive_file(version: &str) -> String {
    format!("{}.{}", archive_root_dir(version), archive_extension())
}

pub fn archive_root_dir(version: &str) -> String {
    format!("node-v{}-{}-{}", version, OS, ARCH)
}
