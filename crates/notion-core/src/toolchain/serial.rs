use image::Image;

use notion_fail::{Fallible, ResultExt};

use semver::Version;

#[derive(Serialize, Deserialize)]
pub struct NodeVersion {
    runtime: String,
    npm: String,
}

#[derive(Serialize, Deserialize)]
pub struct Platform {
    #[serde(default)]
    node: Option<NodeVersion>,
    #[serde(default)]
    yarn: Option<String>,
}

impl Platform {
    pub fn into_image(self) -> Fallible<Option<Image>> {
        Ok(match self.node {
            Some(NodeVersion { runtime, npm }) => {
                let node_str = runtime.to_string();
                let node = Version::parse(&runtime).unknown()?;
                let npm_str = npm.to_string();
                let npm = Version::parse(&npm).unknown()?;
                let yarn_str = self.yarn.clone();
                let yarn = if let Some(yarn) = self.yarn {
                    Some(Version::parse(&yarn).unknown()?)
                } else {
                    None
                };

                Some(Image {
                    node,
                    node_str,
                    npm,
                    npm_str,
                    yarn,
                    yarn_str
                })
            }
            None => None
        })
    }
}

impl Image {
    pub fn to_serial(&self) -> Platform {
        Platform {
            node: Some(NodeVersion {
                runtime: self.node_str.clone(),
                npm: self.npm_str.clone(),
            }),
            yarn: self.yarn_str.clone()
        }
    }
}
