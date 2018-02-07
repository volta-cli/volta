use std::path::Path;
use std::fs::{File, rename};

use path;
use node_archive::{Archive, Source, Cached, Remote};
use style::progress_bar;

use failure;

const PUBLIC_NODE_SERVER_ROOT: &'static str = "https://nodejs.org/dist/";

fn public_node_url(version: &str, archive: &str) -> String {
    format!("{}v{}/{}", PUBLIC_NODE_SERVER_ROOT, version, archive)
}

pub fn by_version(version: &str) -> Result<(), failure::Error> {
    let dest = path::node_versions_dir()?;
    let archive_file = path::archive_file(version);

    let cache_file = path::node_cache_dir()?.join(&archive_file);

    if cache_file.is_file() {
        let file = File::open(cache_file)?;
        let source = Cached::load(file)?;
        by_source(&dest, version, source)?;
    } else {
        let url = public_node_url(version, &archive_file);
        // FIXME: pass the cache file path too so it can be tee'ed as it's fetched
        let source = Remote::fetch(&url, &cache_file)?;
        by_source(&dest, version, source)?;
    }
    Ok(())
}

fn by_source<S: Source>(dest: &Path, version: &str, source: S) -> Result<(), failure::Error> {
    let bar = progress_bar(
        &format!("Installing v{}", version),
        source.uncompressed_size().unwrap_or(source.compressed_size()));

    let archive = Archive::new(source, |_, read| {
        bar.inc(read as u64);
    })?;

    archive.unpack(dest)?;

    rename(dest.join(path::archive_root_dir(version)),
           path::node_version_dir(version)?)?;

    bar.finish_and_clear();
    Ok(())
}
