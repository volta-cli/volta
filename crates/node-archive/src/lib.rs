//! This crate provides types for fetching and unpacking Node distribution
//! archives, which is a tarball for Unixes and a zipfile for Windows.

#[cfg(not(windows))]
extern crate tar;
#[cfg(not(windows))]
extern crate flate2;
#[cfg(not(windows))]
mod tarball;

#[cfg(windows)]
extern crate zip as zip_rs;
#[cfg(windows)]
extern crate verbatim;
#[cfg(windows)]
mod zip;

extern crate reqwest;
extern crate tee;
extern crate progress_read;

#[macro_use]
extern crate failure_derive;
extern crate failure;

#[derive(Fail, Debug)]
#[fail(display = "HTTP failure ({})", code)]
pub(crate) struct HttpError {
    code: ::reqwest::StatusCode
}

use std::io::Read;

#[cfg(not(windows))]
pub use std::io::{Read as StreamingMode};

#[cfg(windows)]
pub use std::io::{Seek as StreamingMode};

/// A data source for fetching a Node archive. In Windows, this is required to
/// implement `std::io::Seek` (required to be able to traverse the contents of
/// zip archives) as well as `std::io::Read`; on other platforms it only needs
/// to implement `Read`.
pub trait Source: Read + StreamingMode {
    /// Produces the uncompressed size of the archive in bytes, when available.
    /// In Windows, this is never available and always produces `None`.
    fn uncompressed_size(&self) -> Option<u64>;

    /// Produces the compressed size of the archive in bytes.
    fn compressed_size(&self) -> u64;
}

#[cfg(not(windows))]
pub use tarball::{Archive, Cached, Remote};

#[cfg(windows)]
pub use zip::{Archive, Cached, Remote};

impl Source for Box<Source> {
    fn uncompressed_size(&self) -> Option<u64> {
        (**self).uncompressed_size()
    }

    fn compressed_size(&self) -> u64 {
        (**self).compressed_size()
    }
}
