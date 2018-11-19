//! Provides utilities for operating on the filesystem.

use std::fs::{self, create_dir_all, read_dir, DirEntry, File, Metadata};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use notion_fail::{ExitCode, FailExt, Fallible, NotionFail, ResultExt};

pub fn touch(path: &Path) -> Fallible<File> {
    if !path.is_file() {
        let basedir = path.parent().unwrap();
        create_dir_all(basedir).unknown()?;
        File::create(path).unknown()?;
    }
    File::open(path).unknown()
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
pub(crate) struct PathInternalError;

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

/// Reads a file, if it exists.
pub fn read_file_opt(path: &PathBuf) -> io::Result<Option<String>> {
    let result: io::Result<String> = fs::read_to_string(path);

    match result {
        Ok(string) => Ok(Some(string)),
        Err(error) => match error.kind() {
            ErrorKind::NotFound => Ok(None),
            _ => Err(error),
        },
    }
}

pub fn read_dir_eager(dir: &Path) -> Fallible<impl Iterator<Item = (DirEntry, Metadata)>> {
    Ok(read_dir(dir).unknown()?
        .map(|entry| {
            let entry = entry.unknown()?;
            let metadata = entry.metadata().unknown()?;
            Ok((entry, metadata))
        })
        .collect::<Fallible<Vec<(DirEntry, Metadata)>>>()?
        .into_iter())
}
