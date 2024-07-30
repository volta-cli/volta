//! Provides types and functions for fetching and unpacking a Node installation
//! zip file in Windows operating systems.

use std::fs::{create_dir_all, File};
use std::io::copy;
use std::path::Path;

use crate::ArchiveError;
use progress_read::ProgressRead;
use verbatim::PathExt;
use zip_rs::ZipArchive;

use super::Archive;
use super::Origin;

pub struct Zip {
    compressed_size: u64,
    data: File,
    origin: Origin,
}

impl Zip {
    /// Loads a cached Node zip archive from the specified file.
    pub fn load(source: File) -> Result<Box<dyn Archive>, ArchiveError> {
        let compressed_size = source.metadata()?.len();

        Ok(Box::new(Zip {
            compressed_size,
            data: source,
            origin: Origin::Local,
        }))
    }

    /// Initiate fetching of a Node zip archive from the given URL, returning
    /// a `Remote` data source.
    pub fn fetch(url: &str, cache_file: &Path) -> Result<Box<dyn Archive>, ArchiveError> {
        let (status, _, mut response) = attohttpc::get(url).send()?.split();

        if !status.is_success() {
            return Err(ArchiveError::HttpError(status));
        }

        {
            let mut file = File::create(cache_file)?;
            copy(&mut response, &mut file)?;
        }

        let file = File::open(cache_file)?;
        let compressed_size = file.metadata()?.len();

        Ok(Box::new(Zip {
            compressed_size,
            data: file,
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

        let mut zip = ZipArchive::new(ProgressRead::new(self.data, (), progress))?;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;

            let (is_dir, subpath) = {
                let name = entry.name();

                // Verbatim paths aren't normalized so we have to use correct r"\" separators.
                (
                    name.ends_with('/'),
                    Path::new(&name.replace('/', r"\")).to_path_buf(),
                )
            };

            if is_dir {
                create_dir_all(dest.join(subpath))?;
            } else {
                let mut file = {
                    if let Some(basedir) = subpath.parent() {
                        create_dir_all(dest.join(basedir))?;
                    }
                    File::create(dest.join(subpath))?
                };
                copy(&mut entry, &mut file)?;
            }
        }
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
