use std::fs::{File, rename};
use std::string::ToString;

use path;
use node_archive::{Archive, Cached, Remote, Source};
use style::progress_bar;

use failure;
use semver::Version;

const PUBLIC_NODE_SERVER_ROOT: &'static str = "https://nodejs.org/dist/";

pub struct Installer {
    source: Box<Source>,
    version: Version
}

impl Installer {
    pub fn public(version: Version) -> Result<Self, failure::Error> {
        let archive_file = path::archive_file(&version.to_string());
        let url = format!("{}v{}/{}", PUBLIC_NODE_SERVER_ROOT, version, &archive_file);
        Installer::remote(version, &url)
    }

    pub fn remote(version: Version, url: &str) -> Result<Self, failure::Error> {
        let archive_file = path::archive_file(&version.to_string());
        let cache_file = path::node_cache_dir()?.join(&archive_file);

        if cache_file.is_file() {
            return Installer::cached(version, File::open(cache_file)?);
        }

        // FIXME: save the cache file path too so it can be tee'ed as it's fetched
        Ok(Installer {
            source: Box::new(Remote::fetch(url, &cache_file)?),
            version: version
        })
    }

    pub fn cached(version: Version, file: File) -> Result<Self, failure::Error> {
        Ok(Installer {
            source: Box::new(Cached::load(file)?),
            version: version
        })
    }

    pub fn install(self) -> Result<Version, failure::Error> {
        let dest = path::node_versions_dir()?;
        let bar = progress_bar(
            "Installing",
            &format!("v{}", self.version),
            self.source.uncompressed_size().unwrap_or(self.source.compressed_size()));

        let archive = Archive::new(self.source, |_, read| {
            bar.inc(read as u64);
        })?;

        archive.unpack(&dest)?;

        let version_string = self.version.to_string();
        rename(dest.join(path::archive_root_dir(&version_string)),
            path::node_version_dir(&version_string)?)?;

        bar.finish_and_clear();
        Ok(self.version)
    }
}
