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

/// Reads the full contents of a directory, eagerly extracting each directory entry
/// and its metadata and returning an iterator over them. Returns `Error` if any of
/// these steps fails.
///
/// This function makes it easier to write high level logic for manipulating the
/// contents of directories (map, filter, etc).
///
/// Note that this function allocates an intermediate vector of directory entries to
/// construct the iterator from, so if a directory is expected to be very large, it
/// will allocate temporary data proportional to the number of entries.
pub fn read_dir_eager(dir: &Path) -> Fallible<impl Iterator<Item = (DirEntry, Metadata)>> {
    Ok(read_dir(dir)
        .unknown()?
        .map(|entry| {
            let entry = entry.unknown()?;
            let metadata = entry.metadata().unknown()?;
            Ok((entry, metadata))
        })
        .collect::<Fallible<Vec<(DirEntry, Metadata)>>>()?
        .into_iter())
}
