//! Provides functions for determining the paths of files and directories
//! in a standard Notion layout.

use std::fs;
use std::io;
use std::path::Path;

use notion_fail::{ExitCode, FailExt, Fallible, NotionFail, ResultExt};

cfg_if! {
    if #[cfg(feature = "universal-docs")] {
        #[doc(cfg(unix))]
        mod unix;

        #[doc(cfg(windows))]
        mod windows;

        pub use self::unix::*;
    } else if #[cfg(unix)] {
        mod unix;
        pub use self::unix::*;
    } else {
        mod windows;
        pub use self::windows::*;
    }
}

pub fn node_archive_file(version: &str) -> String {
    format!("{}.{}", node_archive_root_dir(version), archive_extension())
}

pub fn node_archive_root_dir(version: &str) -> String {
    format!("node-v{}-{}-{}", version, OS, ARCH)
}

pub fn yarn_archive_file(version: &str) -> String {
    format!("{}.{}", yarn_archive_root_dir(version), archive_extension())
}

pub fn yarn_archive_root_dir(version: &str) -> String {
    format!("yarn-v{}", version)
}

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Could not create directory {}: {}", dir, error)]
#[notion_fail(code = "FileSystemError")]
pub(crate) struct CreateDirError {
    pub(crate) dir: String,
    pub(crate) error: String,
}

impl CreateDirError {
    pub(crate) fn for_dir(dir: String) -> impl FnOnce(&io::Error) -> CreateDirError {
        move |error| CreateDirError {
            dir,
            error: error.to_string(),
        }
    }
}

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "`path` internal error")]
#[notion_fail(code = "UnknownError")]
pub (crate) struct PathInternalError;

/// This creates the parent directory of the input path, assuming the input path is a file.
pub fn ensure_containing_dir_exists<P: AsRef<Path>>(path: &P) -> Fallible<()> {
    if let Some(dir) = path.as_ref().parent() {
        fs::create_dir_all(dir)
            .with_context(CreateDirError::for_dir(dir.to_string_lossy().to_string()))
    } else {
        // this was called for a file with no parent directory
        throw!(PathInternalError.unknown());
    }
}
