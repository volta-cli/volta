use std::path::Path;
use std::fs::{File, rename};
use indicatif::{ProgressBar, ProgressStyle};
use term_size;

use config;
use node_archive::{Archive, Source};

#[cfg(not(windows))]
use node_archive::tarball::{self as archive, Tarball as ArchiveFormat};

#[cfg(not(windows))]
use std::io::{Read as Streaming};

#[cfg(windows)]
use node_archive::zip::{self as archive, Zip as ArchiveFormat};

#[cfg(windows)]
use std::io::{Seek as Streaming};

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
    let archive_file = config::archive_file(version);

    let cache_file = config::node_cache_dir().unwrap().join(&archive_file);

    if cache_file.is_file() {
        let file = File::open(cache_file).unwrap();
        let source = archive::Cached::load(file).unwrap();
        by_source(dest, version, source);
    } else {
        let url = config::public_node_url(version, &archive_file);
        // FIXME: pass the cache file path too so it can be tee'ed as it's fetched
        let source = archive::Public::fetch(&url, &cache_file).unwrap().unwrap();
        by_source(dest, version, source);
    }
}

fn by_source<S: Source + Streaming>(dest: &Path, version: &str, source: S) {
    let bar = progress_bar(
        &format!("Installing v{}", version),
        source.uncompressed_size().unwrap_or(source.compressed_size()));

    let archive = ArchiveFormat::new(source, |_, read| {
        bar.inc(read as u64);
    }).unwrap();

    archive.unpack(dest).unwrap();

    rename(dest.join(config::archive_root_dir(version)),
           config::node_version_dir(version).unwrap())
        .unwrap();

    bar.finish_and_clear();
}
