use std::collections::BTreeSet;

use crate::version::version_serde;
use node_semver::Version;
use serde::Deserialize;

/// The public Yarn index.
pub struct YarnIndex {
    pub(super) entries: BTreeSet<Version>,
}

#[derive(Deserialize)]
pub struct RawYarnIndex(Vec<RawYarnEntry>);

#[derive(Deserialize)]
pub struct RawYarnEntry {
    /// Yarn releases are given a tag name of the form "v$version" where $version
    /// is the release's version string.
    #[serde(with = "version_serde")]
    pub tag_name: Version,

    /// The GitHub API provides a list of assets. Some Yarn releases don't include
    /// a tarball, so we don't support them and remove them from the set of available
    /// Yarn versions.
    pub assets: Vec<RawYarnAsset>,
}

impl RawYarnEntry {
    /// Is this entry a full release, i.e., does this entry's asset list include a
    /// proper release tarball?
    fn is_full_release(&self) -> bool {
        let release_filename = &format!("yarn-v{}.tar.gz", self.tag_name)[..];
        self.assets
            .iter()
            .any(|raw_asset| raw_asset.name == release_filename)
    }
}

#[derive(Deserialize)]
pub struct RawYarnAsset {
    /// The filename of an asset included in a Yarn GitHub release.
    pub name: String,
}

impl From<RawYarnIndex> for YarnIndex {
    fn from(raw: RawYarnIndex) -> YarnIndex {
        let mut entries = BTreeSet::new();
        for entry in raw.0 {
            if entry.is_full_release() {
                entries.insert(entry.tag_name);
            }
        }
        YarnIndex { entries }
    }
}
