#[cfg(not(windows))]
extern crate tar;
#[cfg(not(windows))]
extern crate flate2;
#[cfg(not(windows))]
mod tarball;

#[cfg(windows)]
extern crate zip as zip_rs;
#[cfg(windows)]
extern crate untss;
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
pub struct HttpError {
    code: ::reqwest::StatusCode
}

use std::io::Read;

#[cfg(not(windows))]
pub use std::io::{Read as StreamingMode};

#[cfg(windows)]
pub use std::io::{Seek as StreamingMode};

pub trait Source: Read + StreamingMode {
    fn uncompressed_size(&self) -> Option<u64>;
    fn compressed_size(&self) -> u64;
}

#[cfg(not(windows))]
pub use tarball::{Archive, Cached, Remote};

#[cfg(windows)]
pub use zip::{Archive, Cached, Remote};
