use super::Source;

use std::io::{self, Read, Seek, SeekFrom, copy};
use std::path::Path;
use std::fs::{File, create_dir_all};

use reqwest;
use progress_read::ProgressRead;
use zip_rs::ZipArchive;
use verbatim::PathExt;

use failure;

pub struct Cached {
    compressed_size: u64,
    source: File
}

impl Cached {
    pub fn load(source: File) -> io::Result<Cached> {
        let compressed_size = source.metadata()?.len();

        Ok(Cached {
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

impl Seek for Cached {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.source.seek(pos)
    }
}

impl Source for Cached {
    fn uncompressed_size(&self) -> Option<u64> {
        None
    }

    fn compressed_size(&self) -> u64 {
        self.compressed_size
    }
}

pub struct Remote {
    cached: Cached
}

impl Remote {
    pub fn fetch(url: &str, cache_file: &Path) -> Result<Remote, failure::Error> {
        let mut response = reqwest::get(url)?;

        if !response.status().is_success() {
            Err(super::HttpError { code: response.status() })?;
        }

        {
            let mut file = File::create(cache_file)?;
            copy(&mut response, &mut file)?;
        }

        Ok(Remote {
            cached: Cached::load(File::open(cache_file)?)?
        })
    }
}

impl Read for Remote {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.cached.read(buf)
    }
}

impl Seek for Remote {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.cached.seek(pos)
    }
}

impl Source for Remote {
    fn uncompressed_size(&self) -> Option<u64> {
        None
    }

    fn compressed_size(&self) -> u64 {
        self.cached.compressed_size
    }
}

pub struct Archive<S: Source + Seek, F: FnMut(&(), usize)> {
    archive: ZipArchive<ProgressRead<S, (), F>>
}

impl<S: Source + Seek, F: FnMut(&(), usize)> Archive<S, F> {
    pub fn new(source: S, callback: F) -> Result<Archive<S, F>, failure::Error> {
        Ok(Archive {
            archive: ZipArchive::new(ProgressRead::new(source, (), callback))?
        })
    }
}

impl<S: Source + Seek, F: FnMut(&(), usize)> Archive<S, F> {
    pub fn unpack(self, dest: &Path) -> Result<(), failure::Error> {
        // Use a verbatim path to avoid the legacy Windows 260 byte path limit.
        let dest: &Path = dest.to_verbatim();

        let mut zip = self.archive;
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
