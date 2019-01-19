//! This crate provides types for fetching and unpacking compressed
//! archives in tarball or zip format.

#![cfg_attr(feature = "universal-docs", feature(doc_cfg))]

#[macro_use]
extern crate cfg_if;

extern crate flate2;
extern crate tar;

mod tarball;

extern crate verbatim;
extern crate zip as zip_rs;

mod zip;

extern crate progress_read;
extern crate reqwest;
extern crate tee;

extern crate failure;
#[macro_use]
extern crate failure_derive;

#[derive(Fail, Debug)]
#[fail(display = "HTTP failure ({})", code)]
pub struct HttpError {
    pub code: ::reqwest::StatusCode,
}

pub use tarball::Tarball;
pub use zip::Zip;

use std::fs::File;
use std::path::Path;

pub trait Archive {
    fn compressed_size(&self) -> u64;
    fn uncompressed_size(&self) -> Option<u64>;

    /// Unpacks the zip archive to the specified destination folder.
    fn unpack(
        self: Box<Self>,
        dest: &Path,
        progress: &mut FnMut(&(), usize),
    ) -> Result<(), failure::Error>;
}

cfg_if! {
    if #[cfg(unix)] {
        /// Load an archive in the native OS-preferred format from the specified file.
        ///
        /// On Windows, the preferred format is zip. On Unixes, the preferred format
        /// is tarball.
        pub fn load_native(source: File) -> Result<Box<Archive>, failure::Error> {
            Tarball::load(source)
        }

        /// Fetch a remote archive in the native OS-preferred format from the specified
        /// URL and store its results at the specified file path.
        ///
        /// On Windows, the preferred format is zip. On Unixes, the preferred format
        /// is tarball.
        pub fn fetch_native(url: &str, cache_file: &Path) -> Result<Box<Archive>, failure::Error> {
            Tarball::fetch(url, cache_file)
        }
    } else if #[cfg(windows)] {
        /// Load an archive in the native OS-preferred format from the specified file.
        ///
        /// On Windows, the preferred format is zip. On Unixes, the preferred format
        /// is tarball.
        pub fn load_native(source: File) -> Result<Box<Archive>, failure::Error> {
            Zip::load(source)
        }

        /// Fetch a remote archive in the native OS-preferred format from the specified
        /// URL and store its results at the specified file path.
        ///
        /// On Windows, the preferred format is zip. On Unixes, the preferred format
        /// is tarball.
        pub fn fetch_native(url: &str, cache_file: &Path) -> Result<Box<Archive>, failure::Error> {
            Zip::fetch(url, cache_file)
        }
    } else {
        compile_error!("Unsupported OS (expected 'unix' or 'windows').");
    }
}
