use image::Image;

use notion_fail::{Fallible, ResultExt};

use semver::Version;

#[derive(Serialize, Deserialize)]
pub struct Platform {
    #[serde(default)]
    node: Option<String>,
    #[serde(default)]
    yarn: Option<String>,
}

impl Platform {
    pub fn into_image(self) -> Fallible<Option<Image>> {
        Ok(match self.node {
            Some(node) => {
                let node_str = node.to_string();
                let node = Version::parse(&node).unknown()?;
                let yarn_str = self.yarn.clone();
                let yarn = if let Some(yarn) = self.yarn {
                    Some(Version::parse(&yarn).unknown()?)
                } else {
                    None
                };

                Some(Image {
                    node,
                    node_str,
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
            node: Some(self.node_str.clone()),
            yarn: self.yarn_str.clone()
        }
    }
}
