extern crate tar;
extern crate flate2;
extern crate zip as zip_rs;
extern crate reqwest;
extern crate tee;
extern crate progress_read;

#[cfg(windows)]
extern crate untss;

#[macro_use]
extern crate failure_derive;
extern crate failure;

use std::io::Read;
use std::path::Path;

pub mod tarball;
pub mod zip;

#[derive(Fail, Debug)]
#[fail(display = "HTTP failure ({})", code)]
pub struct HttpError {
    code: ::reqwest::StatusCode
}

pub trait Source: Read {
    fn uncompressed_size(&self) -> Option<u64>;
    fn compressed_size(&self) -> u64;
}

pub trait Archive {
    fn unpack(self, dest: &Path) -> Result<(), failure::Error>;
}
