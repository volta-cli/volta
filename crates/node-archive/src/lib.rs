extern crate tar;
extern crate flate2;
extern crate zip;
extern crate reqwest;
extern crate progress_read;

use std::io;
use std::io::Read;
use std::path::Path;

pub mod tarball;

pub trait Source: Read {
    fn uncompressed_size(&self) -> Option<u64>;
    fn compressed_size(&self) -> u64;
}

pub trait Archive {
    fn unpack(self, dest: &Path) -> io::Result<()>;
}
