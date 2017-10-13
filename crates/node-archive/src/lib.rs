// needed for `error_chain!` macro
#![recursion_limit = "1024"]

extern crate tar;
extern crate flate2;
extern crate zip as zip_rs;
extern crate reqwest;
extern crate tee;
extern crate progress_read;

#[macro_use]
extern crate error_chain;

use std::io::Read;
use std::path::Path;

pub mod tarball;
pub mod zip;

mod errors {
    error_chain! {
        foreign_links {
            Reqwest(::reqwest::Error);
            Io(::std::io::Error);
            Zip(::zip_rs::result::ZipError);
        }

        errors {
            HttpFailure(code: ::reqwest::StatusCode) {
                description("HTTP failure"),
                display("HTTP failure ({})", code)
            }
        }
    }
}

pub use errors::*;

pub trait Source: Read {
    fn uncompressed_size(&self) -> Option<u64>;
    fn compressed_size(&self) -> u64;
}

pub trait Archive {
    fn unpack(self, dest: &Path) -> ::Result<()>;
}
