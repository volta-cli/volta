use super::{Archive, Source};

use std::io::{Read, Seek, SeekFrom};
use std::io;
use std::path::Path;
use std::fs::File;

use flate2::read::GzDecoder;
use reqwest::header::{AcceptRanges, ContentLength, Range, RangeUnit, ByteRangeSpec};
use reqwest::Response;
use reqwest;
use tar;
use tee::TeeReader;
use progress_read::ProgressRead;

pub struct Cached {
    uncompressed_size: u64,
    compressed_size: u64,
    source: File
}

impl Cached {
    pub fn load(mut source: File) -> ::Result<Cached> {
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

pub struct Public {
    uncompressed_size: Option<u64>,
    compressed_size: u64,
    source: TeeReader<reqwest::Response, File>
}

impl Public {
    pub fn fetch(url: &str, cache_file: &Path) -> ::Result<Public> {
        let uncompressed_size = fetch_uncompressed_size(url);

        let response = reqwest::get(url)?;

        if !response.status().is_success() {
            bail!(::ErrorKind::HttpFailure(response.status()));
        }

        // FIXME: make compressed_size an Option
        let compressed_size = response.headers().get::<ContentLength>().map_or(0, |cl| **cl);

        let file = File::create(cache_file)?;

        let source = TeeReader::new(response, file);

        Ok(Public {
            uncompressed_size,
            compressed_size,
            source
        })
    }
}

impl Read for Public {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.source.read(buf)
    }
}

impl Source for Public {
    fn uncompressed_size(&self) -> Option<u64> {
        self.uncompressed_size
    }

    fn compressed_size(&self) -> u64 {
        self.compressed_size
    }
}

enum ProgressSource<S: Source, F: FnMut(&(), usize)> {
    Uncompressed(u64, ProgressRead<GzDecoder<S>, (), F>),
    Compressed(u64, GzDecoder<ProgressRead<S, (), F>>)
}

impl<S: Source, F: FnMut(&(), usize)> ProgressSource<S, F> {
    fn new(source: S, callback: F) -> io::Result<ProgressSource<S, F>> {
        match source.uncompressed_size() {
            Some(size) => {
                let decoded = GzDecoder::new(source)?;
                Ok(ProgressSource::Uncompressed(size, ProgressRead::new(decoded, (), callback)))
            }
            None => {
                let size = source.compressed_size();
                let progress = ProgressRead::new(source, (), callback);
                Ok(ProgressSource::Compressed(size, GzDecoder::new(progress)?))
            }
        }
    }
}

impl<S: Source, F: FnMut(&(), usize)> Read for ProgressSource<S, F> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            ProgressSource::Uncompressed(_, ref mut source) => source.read(buf),
            ProgressSource::Compressed(_, ref mut source) => source.read(buf)
        }
    }
}

pub struct Tarball<S: Source, F: FnMut(&(), usize)> {
    archive: tar::Archive<ProgressSource<S, F>>
}

impl<S: Source, F: FnMut(&(), usize)> Tarball<S, F> {
    pub fn new(source: S, callback: F) -> ::Result<Tarball<S, F>> {
        Ok(Tarball {
            archive: tar::Archive::new(ProgressSource::new(source, callback)?)
        })
    }
}

impl<S: Source, F: FnMut(&(), usize)> Archive for Tarball<S, F> {
    fn unpack(mut self, dest: &Path) -> ::Result<()> {
        self.archive.unpack(dest)?;
        Ok(())
    }
}

fn headers_only(url: &str) -> reqwest::Result<Option<Response>> {
    let client = reqwest::Client::new()?;
    let response = client.head(url)?
        .send()?;
    if response.status().is_success() {
        Ok(Some(response))
    } else {
        Ok(None)
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

fn unpack_isize(packed: [u8; 4]) -> u64 {
    let unpacked32: u32 =
        ((packed[0] as u32)      ) +
        ((packed[1] as u32) <<  8) +
        ((packed[2] as u32) << 16) +
        ((packed[3] as u32) << 24);

    unpacked32 as u64
}

fn fetch_isize(url: &str, len: u64) -> reqwest::Result<Option<[u8; 4]>> {
    let client = reqwest::Client::new()?;
    let mut response = client.get(url)?
        .header(Range::Bytes(
            vec![ByteRangeSpec::FromTo(len - 4, len - 1)]
        ))
        .send()?;

    // FIXME: propagate Error
    if response.status().is_success() {
        if response.headers().get::<ContentLength>().map(|cl| **cl) == Some(4) {
            let mut buf = [0; 4];
            if response.read_exact(&mut buf).is_ok() {
                return Ok(Some(buf));
            }
        }
    }

    Ok(None)
}

fn load_isize(file: &mut File) -> io::Result<[u8; 4]> {
    file.seek(SeekFrom::End(-4))?;
    let mut buf = [0; 4];
    file.read_exact(&mut buf)?;
    file.seek(SeekFrom::Start(0))?;
    Ok(buf)
}

fn fetch_uncompressed_size(url: &str) -> Option<u64> {
    let response = match headers_only(url) {
        Ok(Some(response)) => response,
        _ => { return None; }
    };

    if !response.headers().get::<AcceptRanges>()
        .map(|v| v.iter().any(|unit| *unit == RangeUnit::Bytes))
        .unwrap_or(false) {
        return None;
    }

    if let Some(len) = response.headers().get::<ContentLength>().map(|cl| **cl) {
        if let Ok(Some(packed)) = fetch_isize(url, len) {
            return Some(unpack_isize(packed));
        }
    }

    None
}

fn load_uncompressed_size(file: &mut File) -> io::Result<u64> {
    let packed = load_isize(file)?;
    Ok(unpack_isize(packed))
}
