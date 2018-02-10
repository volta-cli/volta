use super::super::manifest;

use failure;
use semver::VersionReq;

use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,

    #[serde(default)]
    pub dependencies: HashMap<String, String>,

    #[serde(default)]
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: HashMap<String, String>,

    pub notion: Option<NotionManifest>
}

#[derive(Serialize, Deserialize)]
pub struct NotionManifest {
    pub node: String,
    pub yarn: Option<String>
}

impl Manifest {
    pub fn into_manifest(self) -> Result<Option<manifest::Manifest>, failure::Error> {
        if let Some(notion) = self.notion {
            return Ok(Some(manifest::Manifest {
                node: parse_version_req(&notion.node)?,
                yarn: if let Some(yarn) = notion.yarn {
                    Some(parse_version_req(&yarn)?)
                } else {
                    None
                },
                dependencies: self.dependencies
            }));
        }

        Ok(None)
    }
}

fn parse_version_req(src: &str) -> Result<VersionReq, failure::Error> {
    let src = src.trim();
    Ok(if src.len() > 0 && src.chars().next().unwrap().is_digit(10) {
        let defaulted = format!("={}", src);
        VersionReq::parse(&defaulted)?
    } else {
        VersionReq::parse(src)?
    })
}
