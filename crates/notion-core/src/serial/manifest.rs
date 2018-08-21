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

    pub toolchain: Option<ToolchainManifest>,
}

#[derive(Serialize, Deserialize)]
pub struct ToolchainManifest {
    pub node: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yarn: Option<String>,
}

impl Manifest {
    pub fn into_manifest(self) -> Fallible<manifest::Manifest> {
        Ok(manifest::Manifest {
            toolchain: self.into_toolchain_manifest()?,
            dependencies: self.dependencies,
            dev_dependencies: self.dev_dependencies,
        })
    }

    pub fn into_toolchain_manifest(&self) -> Fallible<Option<manifest::ToolchainManifest>> {
        if let Some(toolchain) = &self.toolchain {
            return Ok(Some(manifest::ToolchainManifest {
                node: parse_requirements(&toolchain.node)?,
                node_str: toolchain.node.clone(),
                yarn: if let Some(yarn) = &toolchain.yarn {
                    Some(parse_requirements(&yarn)?)
                } else {
                    None
                },
                yarn_str: toolchain.yarn.clone(),
            }));
        }
        Ok(None)
    }
}

impl ToolchainManifest {
    pub fn new(node_version: String, yarn_version: Option<String>) -> Self {
        ToolchainManifest {
            node: node_version,
            yarn: yarn_version,
        }
    }
}
