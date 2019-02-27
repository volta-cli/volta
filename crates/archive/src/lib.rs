//! This crate provides types for fetching and unpacking compressed
//! archives in tarball or zip format.

#![cfg_attr(feature = "universal-docs", feature(doc_cfg))]

mod tarball;
mod zip;

use failure::Fail;

#[derive(Fail, Debug)]
#[fail(display = "HTTP failure ({})", code)]
pub struct HttpError {
    pub code: ::reqwest::StatusCode,
}

pub use crate::tarball::Tarball;
pub use crate::zip::Zip;

use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::path::Path;

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum Action {
    Fetching,
    Unpacking,
}

impl Action {
    /// The maximum width of the displayed Action strings, used for formatting
    pub const MAX_WIDTH: usize = 10;
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            &Action::Fetching => "Fetching",
            &Action::Unpacking => "Unpacking",
        };
        f.write_str(s)
    }
}

pub trait Archive {
    fn compressed_size(&self) -> u64;
    fn uncompressed_size(&self) -> Option<u64>;

    /// Unpacks the zip archive to the specified destination folder.
    fn unpack(
        self: Box<Self>,
        dest: &Path,
        progress: &mut FnMut(&(), usize),
    ) -> Result<(), failure::Error>;

    fn action(&self) -> Action;
}

cfg_if::cfg_if! {
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
