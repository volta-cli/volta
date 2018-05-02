use super::super::manifest;
use super::version::parse_requirements;

use notion_fail::Fallible;

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

    pub notion: Option<NotionManifest>,
}

#[derive(Serialize, Deserialize)]
pub struct NotionManifest {
    pub node: String,
    pub yarn: Option<String>,
}

impl Manifest {
    pub fn into_manifest(self) -> Fallible<Option<manifest::Manifest>> {
        if let Some(notion) = self.notion {
            return Ok(Some(manifest::Manifest {
                node: parse_requirements(&notion.node)?,
                yarn: if let Some(yarn) = notion.yarn {
                    Some(parse_requirements(&yarn)?)
                } else {
                    None
                },
                dependencies: self.dependencies,
            }));
        }

        Ok(None)
    }
}
