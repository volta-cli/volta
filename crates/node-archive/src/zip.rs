use super::{Archive, Source};

use std::io::{self, Read, Seek, SeekFrom, copy};
use std::path::Path;
use std::fs::{File, create_dir_all};

use reqwest;
use progress_read::ProgressRead;
use zip_rs::ZipArchive;

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

pub struct Public {
    cached: Cached
}

impl Public {
    pub fn fetch(url: &str, cache_file: &Path) -> ::Result<Public> {
        let mut response = reqwest::get(url)?;

        if !response.status().is_success() {
            bail!(::ErrorKind::HttpFailure(response.status()));
        }

        {
            let mut file = File::create(cache_file)?;
            copy(&mut response, &mut file)?;
        }

        Ok(Public {
            cached: Cached::load(File::open(cache_file)?)?
        })
    }
}

impl Read for Public {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.cached.read(buf)
    }
}

impl Seek for Public {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.cached.seek(pos)
    }
}

impl Source for Public {
    fn uncompressed_size(&self) -> Option<u64> {
        None
    }

    fn compressed_size(&self) -> u64 {
        self.cached.compressed_size
    }
}

pub struct Zip<S: Source + Seek, F: FnMut(&(), usize)> {
    archive: ZipArchive<ProgressRead<S, (), F>>
}

impl<S: Source + Seek, F: FnMut(&(), usize)> Zip<S, F> {
    pub fn new(source: S, callback: F) -> ::Result<Zip<S, F>> {
        Ok(Zip {
            archive: ZipArchive::new(ProgressRead::new(source, (), callback))?
        })
    }
}

impl<S: Source + Seek, F: FnMut(&(), usize)> Archive for Zip<S, F> {
    fn unpack(self, dest: &Path) -> ::Result<()> {
        let mut zip = self.archive;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            //println!("name: {:?}", entry.name());
            if entry.name().ends_with('/') {
                create_dir_all(dest.join(Path::new(entry.name())))?;
            } else {
                let mut file = {
                    let path = Path::new(entry.name());
                    if let Some(basedir) = path.parent() {
                        create_dir_all(dest.join(basedir))?;
                    }
                    File::create(dest.join(path))?
                };
                copy(&mut entry, &mut file)?;
            }
        }
        Ok(())
    }
}
