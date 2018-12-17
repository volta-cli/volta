use image::Image;

use distro;
use notion_fail::{Fallible, ResultExt};

use semver::Version;
use toml;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NodeVersion {
    pub runtime: String,
    pub npm: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Platform {
    #[serde(default)]
    pub yarn: Option<String>,
    #[serde(default)]
    pub node: Option<NodeVersion>,
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
            None => None,
        })
    }

    /// Deserialize the input TOML String into a Platform
    pub fn from_toml(src: String) -> Fallible<Self> {
        toml::from_str(&src).unknown()
    }

    /// Serialize the Platform to a TOML String
    pub fn to_toml(self) -> Fallible<String> {
        toml::to_string_pretty(&self).unknown()
    }
}

impl Image {
    pub fn to_serial(&self) -> Platform {
        Platform {
            yarn: self.yarn.as_ref().map(|yarn| yarn.to_string()),
            node: Some(NodeVersion {
                runtime: self.node.runtime.to_string(),
                npm: self.node.npm.to_string(),
            }),
        }
    }
}


#[cfg(test)]
pub mod tests {

    use super::*;
    use distro;
    use image;
    use semver;

    const BASIC_TOML_STR: &'static str = r#"yarn = '1.2.3'

[node]
runtime = '4.5.6'
npm = '7.8.9'
"#;

    #[test]
    fn test_from_toml() {
        let toml_str = BASIC_TOML_STR.to_string();
        let platform = Platform::from_toml(toml_str).expect("could not parse TOML string");
        let expected_platform = Platform {
            yarn: Some("1.2.3".to_string()),
            node: Some(NodeVersion {
                runtime: "4.5.6".to_string(),
                npm: "7.8.9".to_string(),
            }),
        };
        assert_eq!(platform, expected_platform);
    }

    #[test]
    fn test_to_toml() {
        let platform = image::Image {
            yarn: Some(semver::Version::parse("1.2.3").expect("could not parse semver version")),
            node: distro::node::NodeVersion {
                runtime: semver::Version::parse("4.5.6").expect("could not parse semver version"),
                npm: semver::Version::parse("7.8.9").expect("could not parse semver version"),
            },
        };
        let toml_str = platform.to_serial().to_toml().expect("could not serialize platform to TOML");
        let expected_toml_str = BASIC_TOML_STR.to_string();
        assert_eq!(toml_str, expected_toml_str);
    }
}
