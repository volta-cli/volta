//! Provides utilities for operating on the filesystem.

use std::fs::{self, create_dir_all, read_dir, DirEntry, File, Metadata};
use std::io;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::error::{Context, ErrorKind, Fallible};
use crate::layout::volta_home;
use retry::delay::Fibonacci;
use retry::{retry, OperationResult};
use tempfile::{tempdir_in, NamedTempFile, TempDir};

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

/// Removes the target directory, if it exists. If the directory doesn't exist, that is treated as
/// success.
pub fn remove_dir_if_exists<P: AsRef<Path>>(path: P) -> Fallible<()> {
    fs::remove_dir_all(&path)
        .or_else(ok_if_not_found)
        .with_context(|| ErrorKind::DeleteDirectoryError {
            directory: path.as_ref().to_owned(),
        })
}

/// Removes the target file, if it exists. If the file doesn't exist, that is treated as success.
pub fn remove_file_if_exists<P: AsRef<Path>>(path: P) -> Fallible<()> {
    fs::remove_file(&path)
        .or_else(ok_if_not_found)
        .with_context(|| ErrorKind::DeleteFileError {
            file: path.as_ref().to_owned(),
        })
}

/// Converts a failure because of file not found into a success.
///
/// Handling the error is preferred over checking if a file exists before removing it, since
/// that avoids a potential race condition between the check and the removal.
pub fn ok_if_not_found<T: Default>(err: io::Error) -> io::Result<T> {
    match err.kind() {
        io::ErrorKind::NotFound => Ok(T::default()),
        _ => Err(err),
    }
}

/// Reads a file, if it exists.
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Option<String>> {
    let result: io::Result<String> = fs::read_to_string(path);

    match result {
        Ok(string) => Ok(Some(string)),
        Err(error) => match error.kind() {
            io::ErrorKind::NotFound => Ok(None),
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
    let tmp_dir = volta_home()?.tmp_dir();
    NamedTempFile::new_in(tmp_dir).with_context(|| ErrorKind::CreateTempFileError {
        in_dir: tmp_dir.to_owned(),
    })
}

/// Creates a staging directory in the Volta tmp directory
pub fn create_staging_dir() -> Fallible<TempDir> {
    let tmp_root = volta_home()?.tmp_dir();
    tempdir_in(tmp_root).with_context(|| ErrorKind::CreateTempDirError {
        in_dir: tmp_root.to_owned(),
    })
}

/// Create a file symlink. The `dst` path will be a symbolic link pointing to the `src` path.
pub fn symlink_file<S, D>(src: S, dest: D) -> io::Result<()>
where
    S: AsRef<Path>,
    D: AsRef<Path>,
{
    #[cfg(windows)]
    return std::os::windows::fs::symlink_file(src, dest);

    #[cfg(unix)]
    return std::os::unix::fs::symlink(src, dest);
}

/// Create a directory symlink. The `dst` path will be a symbolic link pointing to the `src` path
pub fn symlink_dir<S, D>(src: S, dest: D) -> io::Result<()>
where
    S: AsRef<Path>,
    D: AsRef<Path>,
{
    #[cfg(windows)]
    return junction::create(src, dest);

    #[cfg(unix)]
    return std::os::unix::fs::symlink(src, dest);
}

/// Ensure that a given file has 'executable' permissions, otherwise we won't be able to call it
#[cfg(unix)]
pub fn set_executable(bin: &Path) -> io::Result<()> {
    let mut permissions = fs::metadata(bin)?.permissions();
    let mode = permissions.mode();

    if mode & 0o111 != 0o111 {
        permissions.set_mode(mode | 0o111);
        fs::set_permissions(bin, permissions)
    } else {
        Ok(())
    }
}

/// Ensure that a given file has 'executable' permissions, otherwise we won't be able to call it
///
/// Note: This is a no-op on Windows, which has no concept of 'executable' permissions
#[cfg(windows)]
pub fn set_executable(_bin: &Path) -> io::Result<()> {
    Ok(())
}

/// Rename a file or directory to a new name, retrying if the operation fails because of permissions
///
/// Will retry for ~30 seconds with longer and longer delays between each, to allow for virus scan
/// and other automated operations to complete.
pub fn rename<F, T>(from: F, to: T) -> io::Result<()>
where
    F: AsRef<Path>,
    T: AsRef<Path>,
{
    // 21 Fibonacci steps starting at 1 ms is ~28 seconds total
    // See https://github.com/rust-lang/rustup/pull/1873 where this was used by Rustup to work around
    // virus scanning file locks
    let from = from.as_ref();
    let to = to.as_ref();

    retry(Fibonacci::from_millis(1).take(21), || {
        match fs::rename(from, to) {
            Ok(_) => OperationResult::Ok(()),
            Err(e) => match e.kind() {
                io::ErrorKind::PermissionDenied => OperationResult::Retry(e),
                _ => OperationResult::Err(e),
            },
        }
    })
    .map_err(|e| e.error)
}
