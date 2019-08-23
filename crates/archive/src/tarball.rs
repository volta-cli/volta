//! Provides types and functions for fetching and unpacking a Node installation
//! tarball in Unix operating systems.

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use failure::{self, Fail};
use flate2::read::GzDecoder;
use headers_011::Headers011;
use progress_read::ProgressRead;
use reqwest;
use reqwest::hyper_011::header::{AcceptRanges, ByteRangeSpec, ContentLength, Range, RangeUnit};
use reqwest::Response;
use tar;
use tee::TeeReader;

use super::Archive;
use super::Origin;

/// A Node installation tarball.
pub struct Tarball {
    compressed_size: u64,
    // Some servers don't return the right data for byte range queries, so
    // getting the uncompressed archive size for tarballs is an Option.
    // If the uncompressed size is not available, the compressed size will be
    // used for the download/unpack progress indicator, so that will be slightly off.
    uncompressed_size: Option<u64>,
    data: Box<Read>,
    origin: Origin,
}

/// Thrown when the containing directory could not be determined
#[derive(Fail, Debug)]
#[fail(display = "Could not determine directory information for {}", path)]
struct ContainingDirError {
    path: String,
}

/// Thrown when the containing directory could not be determined
#[derive(Fail, Debug)]
#[fail(display = "Could not create directory {}", dir)]
struct CreateDirError {
    dir: String,
}

#[derive(Fail, Debug)]
#[fail(display = "HTTP header '{}' not found", header)]
struct MissingHeaderError {
    header: String,
}

/// This creates the parent directory of the input path, assuming the input path is a file.
pub fn ensure_containing_dir_exists<P: AsRef<Path>>(path: &P) -> Result<(), failure::Error> {
    // TODO: don't know WTF I'm doing with this
    // path.as_ref()
    //     .parent()
    //     .ok_or(Err("some error"))
    //     .and_then(|dir| fs::create_dir_all(dir))
    //     .or(Err("another error"))
    fs::create_dir_all(path.as_ref()).map_err(|_| {
        CreateDirError {
            dir: path.as_ref().to_string_lossy().to_string(),
        }
        .into()
    })
}

/// Determines the length of an HTTP response's content in bytes, using
/// the HTTP `"Content-Length"` header.
fn content_length(response: &Response) -> Result<u64, failure::Error> {
    response
        .headers()
        .get_011::<ContentLength>()
        .map(|v| v.0)
        .ok_or_else(|| {
            MissingHeaderError {
                header: String::from("Content-Length"),
            }
            .into()
        })
}

impl Tarball {
    /// Loads a tarball from the specified file.
    pub fn load(mut source: File) -> Result<Box<Archive>, failure::Error> {
        let uncompressed_size = load_uncompressed_size(&mut source);
        let compressed_size = source.metadata()?.len();
        Ok(Box::new(Tarball {
            uncompressed_size,
            compressed_size,
            data: Box::new(source),
            origin: Origin::Local,
        }))
    }

    /// Initiate fetching of a tarball from the given URL, returning a
    /// tarball that can be streamed (and that tees its data to a local
    /// file as it streams).
    pub fn fetch(url: &str, cache_file: &Path) -> Result<Box<Archive>, failure::Error> {
        let response = reqwest::get(url)?;

        if !response.status().is_success() {
            Err(super::HttpError {
                code: response.status(),
            })?;
        }

        let compressed_size = content_length(&response)?;
        let uncompressed_size = match accepts_byte_ranges(&response) {
            true => fetch_uncompressed_size(url, compressed_size),
            false => None,
        };

        ensure_containing_dir_exists(&cache_file)?;
        let file = File::create(cache_file)?;
        let data = Box::new(TeeReader::new(response, file));

        Ok(Box::new(Tarball {
            uncompressed_size,
            compressed_size,
            data,
            origin: Origin::Remote,
        }))
    }
}

impl Archive for Tarball {
    fn compressed_size(&self) -> u64 {
        self.compressed_size
    }
    fn uncompressed_size(&self) -> Option<u64> {
        self.uncompressed_size
    }
    fn unpack(
        self: Box<Self>,
        dest: &Path,
        progress: &mut FnMut(&(), usize),
    ) -> Result<(), failure::Error> {
        let decoded = GzDecoder::new(self.data);
        let mut tarball = tar::Archive::new(ProgressRead::new(decoded, (), progress));
        tarball.unpack(dest)?;
        Ok(())
    }
    fn origin(&self) -> Origin {
        self.origin
    }
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
    let unpacked32: u32 = (packed[0] as u32)
        + ((packed[1] as u32) << 8)
        + ((packed[2] as u32) << 16)
        + ((packed[3] as u32) << 24);

    unpacked32 as u64
}

#[derive(Fail, Debug)]
#[fail(display = "unexpected content length in HTTP response: {}", length)]
struct UnexpectedContentLengthError {
    length: u64,
}

/// Fetches just the `isize` field (the field that indicates the uncompressed size)
/// of a gzip file from a URL. This makes two round-trips to the server but avoids
/// downloading the entire gzip file. For very small files it's unlikely to be
/// more efficient than simply downloading the entire file up front.
fn fetch_isize(url: &str, len: u64) -> Result<[u8; 4], failure::Error> {
    let client = reqwest::Client::new();
    let mut response = client
        .get(url)
        .header_011(Range::Bytes(vec![ByteRangeSpec::FromTo(len - 4, len - 1)]))
        .send()?;

    if !response.status().is_success() {
        Err(super::HttpError {
            code: response.status(),
        })?;
    }

    let actual_length = content_length(&response)?;

    if actual_length != 4 {
        Err(UnexpectedContentLengthError {
            length: actual_length,
        })?;
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

fn accepts_byte_ranges(response: &Response) -> bool {
    response
        .headers()
        .get_011::<AcceptRanges>()
        .map(|v| v.iter().any(|unit| *unit == RangeUnit::Bytes))
        .unwrap_or(false)
}

/// Determines the uncompressed size of a gzip file hosted at the specified
/// URL by fetching just the metadata associated with the file. This makes
/// an extra round-trip to the server, so it's only more efficient than just
/// downloading the file if the file is large enough that downloading it is
/// slower than the extra round trips.
fn fetch_uncompressed_size(url: &str, len: u64) -> Option<u64> {
    // if there is an error, we ignore it and return None, instead of failing
    fetch_isize(url, len)
        .ok()
        .map(|packed| unpack_isize(packed))
}

/// Determines the uncompressed size of the specified gzip file on disk.
fn load_uncompressed_size(file: &mut File) -> Option<u64> {
    // if there is an error, we ignore it and return None, instead of failing
    load_isize(file).ok().map(|packed| unpack_isize(packed))
}

#[cfg(test)]
pub mod tests {

    use crate::tarball::Tarball;
    use std::fs::File;
    use std::path::PathBuf;

    fn fixture_path(fixture_dir: &str) -> PathBuf {
        let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_manifest_dir.push("fixtures");
        cargo_manifest_dir.push(fixture_dir);
        cargo_manifest_dir
    }

    #[test]
    fn test_load() {
        let mut test_file_path = fixture_path("tarballs");
        test_file_path.push("test-file.tar.gz");
        let test_file = File::open(test_file_path).expect("Couldn't open test file");
        let tarball = Tarball::load(test_file).expect("Failed to load tarball");

        assert_eq!(tarball.uncompressed_size(), Some(10240));
        assert_eq!(tarball.compressed_size(), 402);
    }
}
