//! Provides types and functions for fetching and unpacking a Node installation
//! tarball in Unix operating systems.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use super::{Archive, ArchiveError, Origin};
use attohttpc::header::HeaderMap;
use flate2::read::GzDecoder;
use fs_utils::ensure_containing_dir_exists;
use headers::{AcceptRanges, ContentLength, Header, HeaderMapExt, Range};
use progress_read::ProgressRead;
use tee::TeeReader;

/// A Node installation tarball.
pub struct Tarball {
    compressed_size: u64,
    // Some servers don't return the right data for byte range queries, so
    // getting the uncompressed archive size for tarballs is an Option.
    // If the uncompressed size is not available, the compressed size will be
    // used for the download/unpack progress indicator, so that will be slightly off.
    uncompressed_size: Option<u64>,
    data: Box<dyn Read>,
    origin: Origin,
}

/// Determines the length of an HTTP response's content in bytes, using
/// the HTTP `"Content-Length"` header.
fn content_length(headers: &HeaderMap) -> Result<u64, ArchiveError> {
    headers
        .typed_get::<ContentLength>()
        .map(|v| v.0)
        .ok_or_else(|| ArchiveError::MissingHeaderError(ContentLength::name()))
}

impl Tarball {
    /// Loads a tarball from the specified file.
    pub fn load(mut source: File) -> Result<Box<dyn Archive>, ArchiveError> {
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
    pub fn fetch(url: &str, cache_file: &Path) -> Result<Box<dyn Archive>, ArchiveError> {
        let (status, headers, response) = attohttpc::get(url).send()?.split();

        if !status.is_success() {
            return Err(ArchiveError::HttpError(status));
        }

        let compressed_size = content_length(&headers)?;
        let uncompressed_size = if accepts_byte_ranges(&headers) {
            fetch_uncompressed_size(url, compressed_size)
        } else {
            None
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
        progress: &mut dyn FnMut(&(), usize),
    ) -> Result<(), ArchiveError> {
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

/// Fetches just the `isize` field (the field that indicates the uncompressed size)
/// of a gzip file from a URL. This makes two round-trips to the server but avoids
/// downloading the entire gzip file. For very small files it's unlikely to be
/// more efficient than simply downloading the entire file up front.
fn fetch_isize(url: &str, len: u64) -> Result<[u8; 4], ArchiveError> {
    let (status, headers, mut response) = {
        let mut request = attohttpc::get(url);
        request
            .headers_mut()
            .typed_insert(Range::bytes(len - 4..len).unwrap());
        request.send()?.split()
    };

    if !status.is_success() {
        return Err(ArchiveError::HttpError(status));
    }

    let actual_length = content_length(&headers)?;

    if actual_length != 4 {
        return Err(ArchiveError::UnexpectedContentLengthError(actual_length));
    }

    let mut buf = [0; 4];
    response.read_exact(&mut buf)?;
    Ok(buf)
}

/// Loads the `isize` field (the field that indicates the uncompressed size)
/// of a gzip file from disk.
fn load_isize(file: &mut File) -> Result<[u8; 4], ArchiveError> {
    file.seek(SeekFrom::End(-4))?;
    let mut buf = [0; 4];
    file.read_exact(&mut buf)?;
    file.seek(SeekFrom::Start(0))?;
    Ok(buf)
}

fn accepts_byte_ranges(headers: &HeaderMap) -> bool {
    headers
        .typed_get::<AcceptRanges>()
        .is_some_and(|v| v == AcceptRanges::bytes())
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
        .map(|s| u32::from_le_bytes(s) as u64)
}

/// Determines the uncompressed size of the specified gzip file on disk.
fn load_uncompressed_size(file: &mut File) -> Option<u64> {
    // if there is an error, we ignore it and return None, instead of failing
    load_isize(file).ok().map(|s| u32::from_le_bytes(s) as u64)
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
