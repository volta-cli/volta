//! Provides types and functions for fetching and unpacking a Node installation
//! tarball in Unix operating systems.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use failure;
use flate2::read::GzDecoder;
use progress_read::ProgressRead;
use reqwest;
use reqwest::header::{AcceptRanges, ByteRangeSpec, ContentLength, Range, RangeUnit};
use reqwest::Response;
use tar;
use tee::TeeReader;

use super::Archive;

/// A Node installation tarball.
pub struct Tarball {
    compressed_size: u64,
    uncompressed_size: u64,
    data: Box<Read>,
}

#[derive(Fail, Debug)]
#[fail(display = "HTTP header '{}' not found", header)]
struct MissingHeaderError {
    header: String,
}

/// Determines the length of an HTTP response's content in bytes, using
/// the HTTP `"Content-Length"` header.
fn content_length(response: &Response) -> Result<u64, failure::Error> {
    Ok(match response.headers().get::<ContentLength>() {
        Some(content_length) => **content_length,
        None => {
            return Err(MissingHeaderError {
                header: String::from("Content-Length"),
            }
            .into());
        }
    })
}

impl Tarball {
    /// Loads a tarball from the specified file.
    pub fn load(mut source: File) -> Result<Box<Archive>, failure::Error> {
        let uncompressed_size = load_uncompressed_size(&mut source)?;
        let compressed_size = source.metadata()?.len();
        Ok(Box::new(Tarball {
            uncompressed_size,
            compressed_size,
            data: Box::new(source),
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

        ensure_accepts_byte_ranges(&response)?;

        let uncompressed_size = fetch_uncompressed_size(url, compressed_size)?;

        let file = File::create(cache_file)?;
        let data = Box::new(TeeReader::new(response, file));

        Ok(Box::new(Tarball {
            uncompressed_size,
            compressed_size,
            data,
        }))
    }
}

impl Archive for Tarball {
    fn compressed_size(&self) -> u64 {
        self.compressed_size
    }
    fn uncompressed_size(&self) -> Option<u64> {
        Some(self.uncompressed_size)
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
    let client = reqwest::Client::new()?;
    let mut response = client
        .get(url)?
        .header(Range::Bytes(vec![ByteRangeSpec::FromTo(len - 4, len - 1)]))
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

#[derive(Fail, Debug)]
#[fail(display = "HTTP server does not accept byte range requests")]
struct ByteRangesNotAcceptedError;

fn ensure_accepts_byte_ranges(response: &Response) -> Result<(), failure::Error> {
    if !response
        .headers()
        .get::<AcceptRanges>()
        .map(|v| v.iter().any(|unit| *unit == RangeUnit::Bytes))
        .unwrap_or(false)
    {
        Err(ByteRangesNotAcceptedError)?;
    }
    Ok(())
}

/// Determines the uncompressed size of a gzip file hosted at the specified
/// URL by fetching just the metadata associated with the file. This makes
/// an extra round-trip to the server, so it's only more efficient than just
/// downloading the file if the file is large enough that downloading it is
/// slower than the extra round trips.
fn fetch_uncompressed_size(url: &str, len: u64) -> Result<u64, failure::Error> {
    let packed = fetch_isize(url, len)?;
    Ok(unpack_isize(packed))
}

/// Determines the uncompressed size of the specified gzip file on disk.
fn load_uncompressed_size(file: &mut File) -> Result<u64, failure::Error> {
    let packed = load_isize(file)?;
    Ok(unpack_isize(packed))
}

#[cfg(test)]
pub mod tests {

    use std::fs::File;
    use std::path::PathBuf;
    use tarball::Tarball;

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
