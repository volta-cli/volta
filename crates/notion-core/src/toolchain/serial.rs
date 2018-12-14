use image::Image;

use distro;
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
                let node = distro::node::NodeVersion {
                    runtime: Version::parse(&runtime).unknown()?,
                    npm: Version::parse(&npm).unknown()?,
                };
                let yarn = if let Some(yarn) = self.yarn {
                    Some(Version::parse(&yarn).unknown()?)
                } else {
                    None
                };

                Some(Image { node, yarn })
            }
            None => None
        })
    }
}

impl Image {
    pub fn to_serial(&self) -> Platform {
        Platform {
            node: Some(NodeVersion {
                runtime: self.node.runtime.to_string(),
                npm: self.node.npm.to_string(),
            }),
            yarn: self.yarn.as_ref().map(|yarn| yarn.to_string())
        }
    }
}
