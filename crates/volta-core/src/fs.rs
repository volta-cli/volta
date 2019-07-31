//! Provides utilities for operating on the filesystem.

use std::fs::{self, create_dir_all, read_dir, DirEntry, File, Metadata};
use std::io::{self, ErrorKind};
use std::path::Path;

use crate::error::ErrorDetails;
use crate::path;
use tempfile::{tempdir_in, NamedTempFile, TempDir};
use volta_fail::{Fallible, ResultExt};

/// Opens a file, creating it if it doesn't exist
pub fn touch(path: &Path) -> io::Result<File> {
    if !path.is_file() {
        if let Some(basedir) = path.parent() {
            create_dir_all(basedir)?;
        }
        File::create(path)?;
    }
    File::open(path)
}

/// This creates the parent directory of the input path, assuming the input path is a file.
pub fn ensure_containing_dir_exists<P: AsRef<Path>>(path: &P) -> Fallible<()> {
    path.as_ref()
        .parent()
        .ok_or(
            ErrorDetails::ContainingDirError {
                path: path.as_ref().to_path_buf(),
            }
            .into(),
        )
        .and_then(|dir| {
            fs::create_dir_all(dir).with_context(|_| ErrorDetails::CreateDirError {
                dir: dir.to_path_buf(),
            })
        })
}

/// This deletes the input directory, if it exists
pub fn ensure_dir_does_not_exist<P: AsRef<Path>>(path: &P) -> Fallible<()> {
    if path.as_ref().exists() {
        // remove the directory and all of its contents
        fs::remove_dir_all(path).with_context(delete_dir_error(path))?;
    }
    Ok(())
}

pub fn delete_dir_error<P: AsRef<Path>>(directory: &P) -> impl FnOnce(&io::Error) -> ErrorDetails {
    let directory = directory.as_ref().to_path_buf();
    |_| ErrorDetails::DeleteDirectoryError { directory }
}

pub fn delete_file_error<P: AsRef<Path>>(file: &P) -> impl FnOnce(&io::Error) -> ErrorDetails {
    let file = file.as_ref().to_path_buf();
    |_| ErrorDetails::DeleteFileError { file }
}

/// Reads a file, if it exists.
pub fn read_file_opt<P: AsRef<Path>>(path: P) -> io::Result<Option<String>> {
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
pub fn read_dir_eager(dir: &Path) -> io::Result<impl Iterator<Item = (DirEntry, Metadata)>> {
    let entries = read_dir(dir)?;
    let vec = entries
        .map(|entry| {
            let entry = entry?;
            let metadata = entry.metadata()?;
            Ok((entry, metadata))
        })
        .collect::<io::Result<Vec<(DirEntry, Metadata)>>>()?;

    Ok(vec.into_iter())
}

/// Reads the contents of a directory and returns a Vec of the matched results
/// from the input function
pub fn dir_entry_match<T, F>(dir: &Path, mut f: F) -> io::Result<Vec<T>>
where
    F: FnMut(&DirEntry) -> Option<T>,
{
    let entries = read_dir_eager(dir)?;
    Ok(entries
        .filter(|(_, metadata)| metadata.is_file())
        .filter_map(|(entry, _)| f(&entry))
        .collect::<Vec<T>>())
}

/// Creates a NamedTempFile in the Volta tmp directory
pub fn create_staging_file() -> Fallible<NamedTempFile> {
    let tmp_dir = path::tmp_dir()?;
    NamedTempFile::new_in(&tmp_dir)
        .with_context(|_| ErrorDetails::CreateTempFileError { in_dir: tmp_dir })
}

/// Creates a staging directory in the Volta tmp directory
pub fn create_staging_dir() -> Fallible<TempDir> {
    let tmp_root = path::tmp_dir()?;
    tempdir_in(&tmp_root).with_context(|_| ErrorDetails::CreateTempDirError { in_dir: tmp_root })
}
