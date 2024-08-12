//! Provides types and functions for fetching and unpacking a Node installation
//! zip file in Windows operating systems.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::{content_length, ArchiveError};
use fs_utils::ensure_containing_dir_exists;
use progress_read::ProgressRead;
use tee::TeeReader;
use verbatim::PathExt;
use zip_rs::unstable::stream::ZipStreamReader;

use super::Archive;
use super::Origin;

pub struct Zip {
    compressed_size: u64,
    data: Box<dyn Read>,
    origin: Origin,
}

impl Zip {
    /// Loads a cached Node zip archive from the specified file.
    pub fn load(source: File) -> Result<Box<dyn Archive>, ArchiveError> {
        let compressed_size = source.metadata()?.len();

        Ok(Box::new(Zip {
            compressed_size,
            data: Box::new(source),
            origin: Origin::Local,
        }))
    }

    /// Initiate fetching of a Node zip archive from the given URL, returning
    /// a `Remote` data source.
    pub fn fetch(url: &str, cache_file: &Path) -> Result<Box<dyn Archive>, ArchiveError> {
        let (status, headers, response) = attohttpc::get(url).send()?.split();

        if !status.is_success() {
            return Err(ArchiveError::HttpError(status));
        }

        let compressed_size = content_length(&headers)?;

        ensure_containing_dir_exists(&cache_file)?;
        let file = File::create(cache_file)?;
        let data = Box::new(TeeReader::new(response, file));

        Ok(Box::new(Zip {
            compressed_size,
            data,
            origin: Origin::Remote,
        }))
    }
}

impl Archive for Zip {
    fn compressed_size(&self) -> u64 {
        self.compressed_size
    }
    fn unpack(
        self: Box<Self>,
        dest: &Path,
        progress: &mut dyn FnMut(&(), usize),
    ) -> Result<(), ArchiveError> {
        // Use a verbatim path to avoid the legacy Windows 260 byte path limit.
        let dest: &Path = &dest.to_verbatim();
        let zip = ZipStreamReader::new(ProgressRead::new(self.data, (), progress));
        zip.extract(dest)?;
        Ok(())
    }
    fn origin(&self) -> Origin {
        self.origin
    }
}

#[cfg(test)]
pub mod tests {

    use crate::zip::Zip;
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
        let mut test_file_path = fixture_path("zips");
        test_file_path.push("test-file.zip");
        let test_file = File::open(test_file_path).expect("Couldn't open test file");
        let zip = Zip::load(test_file).expect("Failed to load zip file");

        assert_eq!(zip.compressed_size(), 214);
    }
}
