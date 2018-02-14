use super::Source;

use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::fs::File;

use flate2::read::GzDecoder;
use reqwest::header::{AcceptRanges, ContentLength, Range, RangeUnit, ByteRangeSpec};
use reqwest::Response;
use reqwest;
use tar;
use tee::TeeReader;
use progress_read::ProgressRead;
use failure;

/// A data source for a Node tarball that has been cached to the filesystem.
pub struct Cached {
    uncompressed_size: u64,
    compressed_size: u64,
    source: File
}

impl Cached {

    /// Loads a cached Node tarball from the specified file.
    pub fn load(mut source: File) -> Result<Cached, failure::Error> {
        let uncompressed_size = load_uncompressed_size(&mut source)?;
        let compressed_size = source.metadata()?.len();

        Ok(Cached {
            uncompressed_size,
            compressed_size,
            source
        })
    }

}

impl Read for Cached {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.source.read(buf)
    }

}

impl Source for Cached {
    fn uncompressed_size(&self) -> Option<u64> {
        Some(self.uncompressed_size)
    }

    fn compressed_size(&self) -> u64 {
        self.compressed_size
    }
}

/// A data source for fetching a Node tarball from a remote server.
pub struct Remote {
    uncompressed_size: u64,
    compressed_size: u64,
    source: TeeReader<reqwest::Response, File>
}

#[derive(Fail, Debug)]
#[fail(display = "HTTP header '{}' not found", header)]
struct MissingHeaderError {
    header: String
}

/// Determines the length of an HTTP response's content in bytes, using
/// the HTTP `"Content-Length"` header.
fn content_length(response: &Response) -> Result<u64, failure::Error> {
    Ok(match response.headers().get::<ContentLength>() {
        Some(content_length) => **content_length,
        None => {
            return Err(MissingHeaderError { header: String::from("Content-Length") }.into());
        }
    })
}

impl Remote {

    /// Initiate fetching of a Node tarball from the given URL, returning
    /// a `Remote` data source.
    pub fn fetch(url: &str, cache_file: &Path) -> Result<Remote, failure::Error> {
        let uncompressed_size = fetch_uncompressed_size(url)?;
        let response = reqwest::get(url)?;

        if !response.status().is_success() {
            Err(super::HttpError { code: response.status() })?;
        }

        let compressed_size = content_length(&response)?;
        let file = File::create(cache_file)?;
        let source = TeeReader::new(response, file);

        Ok(Remote {
            uncompressed_size,
            compressed_size,
            source
        })
    }

}

impl Read for Remote {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.source.read(buf)
    }
}

impl Source for Remote {
    fn uncompressed_size(&self) -> Option<u64> {
        Some(self.uncompressed_size)
    }

    fn compressed_size(&self) -> u64 {
        self.compressed_size
    }
}

/// A Node installation tarball.
pub struct Archive<S: Source, F: FnMut(&(), usize)> {
    archive: tar::Archive<ProgressRead<GzDecoder<S>, (), F>>
}

impl<S: Source, F: FnMut(&(), usize)> Archive<S, F> {

    /// Constructs a new `Archive` from the specified data source and with the
    /// specified progress callback.
    pub fn new(source: S, callback: F) -> Result<Archive<S, F>, failure::Error> {
        let decoded = GzDecoder::new(source);
        Ok(Archive {
            archive: tar::Archive::new(ProgressRead::new(decoded, (), callback))
        })
    }

}

impl<S: Source, F: FnMut(&(), usize)> Archive<S, F> {

    /// Unpacks the tarball to the specified destination folder.
    pub fn unpack(mut self, dest: &Path) -> Result<(), failure::Error> {
        self.archive.unpack(dest)?;
        Ok(())
    }

}

/// Fetches just the headers of a URL.
fn headers_only(url: &str) -> Result<Response, failure::Error> {
    let client = reqwest::Client::new()?;
    let response = client.head(url)?.send()?;
    if !response.status().is_success() {
        Err(super::HttpError { code: response.status() })?;
    }
    Ok(response)
}

// From http://www.gzip.org/zlib/rfc-gzip.html#member-format
// 
//   0   1   2   3   4   5   6   7
// +---+---+---+---+---+---+---+---+
// |     CRC32     |     ISIZE     |
// +---+---+---+---+---+---+---+---+
//
// ISIZE (Input SIZE)
//    This contains the size of the original (uncompressed) input data modulo 2^32.

/// Unpacks the `isize` field from a gzip payload as a 64-bit integer.
fn unpack_isize(packed: [u8; 4]) -> u64 {
    let unpacked32: u32 =
        ((packed[0] as u32)      ) +
        ((packed[1] as u32) <<  8) +
        ((packed[2] as u32) << 16) +
        ((packed[3] as u32) << 24);

    unpacked32 as u64
}

#[derive(Fail, Debug)]
#[fail(display = "unexpected content length in HTTP response: {}", length)]
struct UnexpectedContentLengthError {
    length: u64
}

/// Fetches just the `isize` field (the field that indicates the uncompressed size)
/// of a gzip file from a URL. This makes two round-trips to the server but avoids
/// downloading the entire gzip file. For very small files it's unlikely to be
/// more efficient than simply downloading the entire file up front.
fn fetch_isize(url: &str, len: u64) -> Result<[u8; 4], failure::Error> {
    let client = reqwest::Client::new()?;
    let mut response = client.get(url)?
        .header(Range::Bytes(
            vec![ByteRangeSpec::FromTo(len - 4, len - 1)]
        ))
        .send()?;

    if !response.status().is_success() {
        Err(super::HttpError { code: response.status() })?;
    }

    let actual_length = content_length(&response)?;

    if actual_length != 4 {
        Err(UnexpectedContentLengthError { length: actual_length })?;
    }

    let mut buf = [0; 4];
    response.read_exact(&mut buf)?;
    Ok(buf)
}

/// Loads the `isize` field (the field that indicates the uncompressed size)
/// of a gzip file from disk.
fn load_isize(file: &mut File) -> Result<[u8; 4], failure::Error> {
    file.seek(SeekFrom::End(-4))?;
    let mut buf = [0; 4];
    file.read_exact(&mut buf)?;
    file.seek(SeekFrom::Start(0))?;
    Ok(buf)
}

#[derive(Fail, Debug)]
#[fail(display = "HTTP server does not accept byte range requests")]
struct ByteRangesNotAcceptedError;

/// Determines the uncompressed size of a gzip file hosted at the specified
/// URL by fetching just the metadata associated with the file. This makes
/// two round-trips to the server, so it is only more efficient than simply
/// downloading the file if the file is large enough that downloading it is
/// slower than the extra round trips.
fn fetch_uncompressed_size(url: &str) -> Result<u64, failure::Error> {
    let response = headers_only(url)?;

    if !response.headers().get::<AcceptRanges>()
        .map(|v| v.iter().any(|unit| *unit == RangeUnit::Bytes))
        .unwrap_or(false) {
        Err(ByteRangesNotAcceptedError)?;
    }

    let len = content_length(&response)?;
    let packed = fetch_isize(url, len)?;
    Ok(unpack_isize(packed))
}

/// Determines the uncompressed size of the specified gzip file on disk.
fn load_uncompressed_size(file: &mut File) -> Result<u64, failure::Error> {
    let packed = load_isize(file)?;
    Ok(unpack_isize(packed))
}
