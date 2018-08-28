//! Provides types and functions for fetching and unpacking a Node installation
//! zip file in Windows operating systems.

use std::io::{self, Read, Seek, copy};
use std::path::Path;
use std::fs::{File, create_dir_all};

use reqwest;
use progress_read::ProgressRead;
use zip_rs::ZipArchive;
use verbatim::PathExt;

use failure;

use super::Archive;

pub struct Zip<S: Read + Seek> {
    compressed_size: u64,
    data: S
}

impl Zip<File> {

    /// Loads a cached Node zip archive from the specified file.
    pub fn load(source: File) -> io::Result<Self> {
        let compressed_size = source.metadata()?.len();

        Ok(Zip {
            compressed_size,
            data: source
        })
    }

    /// Initiate fetching of a Node zip archive from the given URL, returning
    /// a `Remote` data source.
    pub fn fetch(url: &str, cache_file: &Path) -> Result<Self, failure::Error> {
        let mut response = reqwest::get(url)?;

        if !response.status().is_success() {
            Err(super::HttpError { code: response.status() })?;
        }

        {
            let mut file = File::create(cache_file)?;
            copy(&mut response, &mut file)?;
        }

        let file = File::create(cache_file)?;
        let compressed_size = file.metadata()?.len();

        Ok(Zip {
            compressed_size,
            data: file
        })
    }

}

impl<S: Read + Seek> Archive for Zip<S> {
    fn compressed_size(&self) -> u64 { self.compressed_size }
    fn uncompressed_size(&self) -> Option<u64> { None }
    fn unpack(self: Box<Self>, dest: &Path, progress: &mut FnMut(&(), usize)) -> Result<(), failure::Error> {
        // Use a verbatim path to avoid the legacy Windows 260 byte path limit.
        let dest: &Path = &dest.to_verbatim();

        let mut zip = ZipArchive::new(ProgressRead::new(self.data, (), progress))?;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;

            let (is_dir, subpath) = {
                let name = entry.name();

                // Verbatim paths aren't normalized so we have to use correct r"\" separators.
                (name.ends_with('/'), Path::new(&name.replace('/', r"\")).to_path_buf())
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

}


#[cfg(test)]
pub mod tests {

    use zip::Zip;
    use std::path::PathBuf;
    use std::fs::File;

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

        assert_eq!(zip.compressed_size, 214);
    }
}
