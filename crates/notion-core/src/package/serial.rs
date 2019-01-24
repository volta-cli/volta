// Serialization for npm package information

use std::collections::HashMap;

use notion_fail::Fallible;
use notion_fail::ResultExt;
use semver::Version;


#[derive(Serialize, Deserialize, Debug)]
pub struct PackageMetadata {
    pub name: String,
    pub description: Option<String>,
    pub versions: HashMap<String, PackageInfo>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: PackageDistTags,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageInfo {
    // there's a lot more in there, but right now just care about the version
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageDistTags {
    pub latest: String,
    pub beta: String,
}


impl PackageMetadata {
    pub fn into_versions(self) -> Fallible<super::PackageVersions> {
        let latest = Version::parse(&self.dist_tags.latest).unknown()?;

        let mut entries = Vec::new();
        for (_, info) in self.versions {
            let parsed_version = Version::parse(&info.version).unknown()?;
            entries.push(parsed_version);
        }
        Ok(super::PackageVersions{ latest: latest, entries: entries })
    }
}

