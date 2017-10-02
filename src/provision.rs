use std::path::Path;
use std::fs::rename;
use std::io::Read;

use reqwest;
use reqwest::header::{AcceptRanges, ContentLength, Range, RangeUnit, ByteRangeSpec};
use reqwest::Response;
use flate2::read::GzDecoder;
use tar::Archive;
use indicatif::{ProgressBar, ProgressStyle};
use term_size;

use config;

// FIXME: make the case analysis here complete and rigorous

#[cfg(target_os = "macos")]
const OS: &'static str = "darwin";

#[cfg(target_os = "linux")]
const OS: &'static str = "linux";

#[cfg(target_os = "windows")]
const OS: &'static str = "win";

#[cfg(target_arch = "x86")]
const ARCH: &'static str = "x86";

#[cfg(target_arch = "x86_64")]
const ARCH: &'static str = "x64";

struct ProgressRead<R: Read, T, F: FnMut(&T, usize) -> T> {
    source: R,
    accumulator: T,
    progress: F
}

impl<R: Read, T, F: FnMut(&T, usize) -> T> Read for ProgressRead<R, T, F> {
    fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
        let len = self.source.read(buf)?;
        let new_accumulator = {
            let progress = &mut self.progress;
            progress(&self.accumulator, len)
        };
        self.accumulator = new_accumulator;
        Ok(len)
    }
}

impl<R: Read, T, F: FnMut(&T, usize) -> T> ProgressRead<R, T, F> {
    fn new(source: R, init: T, progress: F) -> ProgressRead<R, T, F> {
        ProgressRead { source, accumulator: init, progress }
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

fn byte_range(url: &str, start: u64, end: u64) -> reqwest::Result<Option<Vec<u8>>> {
    let client = reqwest::Client::new()?;
    let mut response = client.get(url)?
        .header(Range::Bytes(
            vec![ByteRangeSpec::FromTo(start, end)]
        ))
        .send()?;

    let expected_len = (end + 1) - start;
    if expected_len >= (usize::max_value() as u64) {
        panic!("byte range ({}, {}) exceeds system buffer capacity", start, end);
    }

    // FIXME: propagate Error
    if response.status().is_success() {
        let len = response.headers().get::<ContentLength>()
            .map(|cl| **cl)
            .unwrap_or(0);
        if len == expected_len {
            let mut buf = Vec::with_capacity(len as usize);
            if ::std::io::copy(&mut response, &mut buf).is_ok() {
                return Ok(Some(buf));
            }
        }
    }

    Ok(None)
}

fn gunzipped_content_length(url: &str) -> Option<u64> {
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
        if let Ok(Some(bytes)) = byte_range(url, len - 4, len - 1) {
            let gunzipped_len: u32 =
                (bytes[0] as u32) +
                ((bytes[1] as u32) << 8) +
                ((bytes[2] as u32) << 16) +
                ((bytes[3] as u32) << 24);
            return Some(gunzipped_len as u64);
        }
    }

    None
}

fn progress_bar(msg: &str, len: u64) -> ProgressBar {
    let display_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    let msg_width = msg.len();

    // Installing v1.23.4  [====================>                   ]  50%
    // |----------------|   |--------------------------------------|  |-|
    //         msg                           bar                   percentage
    let available_width = display_width - 2 - msg_width - 2 - 2 - 1 - 3 - 1;
    let bar_width = ::std::cmp::min(available_width, 40);

    let bar = ProgressBar::new(len);

    bar.set_message(msg);
    bar.set_style(ProgressStyle::default_bar()
        // FIXME: instead of fixed 40 compute based on console size
        .template(&format!("{{msg}}  [{{bar:{}.cyan/blue}}] {{percent:>3}}%", bar_width))
        .progress_chars("=> "));

    bar
}

// FIXME: return Result
pub fn by_version(dest: &Path, version: &str) {
    let verbose_root = format!("node-v{}-{}-{}", version, OS, ARCH);
    let concise_root = format!("v{}", version);

    let url = config::public_node_url(version, OS, ARCH);

    let uncompressed_len = gunzipped_content_length(&url);

    // FIXME: propagate Result
    let response = reqwest::get(&url).unwrap();

    // FIXME: propagate Result
    if !response.status().is_success() {
        panic!("failed response: {:?}", response.status());
    }

    let compressed_len = response.headers().get::<ContentLength>()
        .map(|cl| **cl)
        .unwrap_or(0);

    /*
    let compressed_len: u64 = 14_001_361;
    let response = ::std::fs::File::open("/home/dherman/Sources/Linux/nemo/node-v6.11.3-linux-x64.tar.gz").unwrap();
    */

    let bar = progress_bar(
        &format!("Installing v{}", version),
        uncompressed_len.unwrap_or(compressed_len));

    // FIXME: propagate Result
    if uncompressed_len.is_some() {
        //println!("computing progress as fraction of uncompressed tarball");
        let tarball = GzDecoder::new(response).unwrap();
        let mut archive = Archive::new(ProgressRead::new(tarball, (), |_, read| {
            bar.inc(read as u64);
        }));
        archive.unpack(dest).unwrap();
    } else {
        //println!("computing progress as fraction of compressed tarball");
        let tarball = GzDecoder::new(ProgressRead::new(response, (), |_, read| {
            bar.inc(read as u64);
        })).unwrap();
        let mut archive = Archive::new(tarball);
        archive.unpack(dest).unwrap();
    }

    rename(dest.join(&verbose_root), dest.join(&concise_root)).unwrap();

    bar.finish_and_clear();
}
