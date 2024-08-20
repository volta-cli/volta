//! Provides types and functions for fetching and unpacking a Node installation
//! tarball in Unix operating systems.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::{content_length, Archive, ArchiveError, Origin};
use flate2::read::GzDecoder;
use fs_utils::ensure_containing_dir_exists;
use progress_read::ProgressRead;
use tee::TeeReader;

/// A Node installation tarball.
pub struct Tarball {
    compressed_size: u64,
    data: Box<dyn Read>,
    origin: Origin,
}

impl Tarball {
    /// Loads a tarball from the specified file.
    pub fn load(source: File) -> Result<Box<dyn Archive>, ArchiveError> {
        let compressed_size = source.metadata()?.len();
        Ok(Box::new(Tarball {
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

        ensure_containing_dir_exists(&cache_file)?;
        let file = File::create(cache_file)?;
        let data = Box::new(TeeReader::new(response, file));

        Ok(Box::new(Tarball {
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
    fn unpack(
        self: Box<Self>,
        dest: &Path,
        progress: &mut dyn FnMut(&(), usize),
    ) -> Result<(), ArchiveError> {
        let decoded = GzDecoder::new(ProgressRead::new(self.data, (), progress));
        let mut tarball = tar::Archive::new(decoded);
        tarball.unpack(dest)?;
        Ok(())
    }
    fn origin(&self) -> Origin {
        self.origin
    }
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

        assert_eq!(tarball.compressed_size(), 402);
    }
}
