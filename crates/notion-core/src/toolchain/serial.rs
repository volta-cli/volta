use platform::PlatformSpec;

use notion_fail::{Fallible, ResultExt};

use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NodeVersion {
    pub runtime: String,
    pub npm: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Platform {
    #[serde(default)]
    pub node: Option<NodeVersion>,
    #[serde(default)]
    pub yarn: Option<String>,
}

impl Platform {
    pub fn into_image(self) -> Fallible<Option<PlatformSpec>> {
        Ok(match self.node {
            Some(NodeVersion { runtime, npm }) => {
                let node_runtime = Version::parse(&runtime).unknown()?;
                let npm = if let Some(npm_version) = npm {
                    Some(Version::parse(&npm_version).unknown()?)
                } else {
                    None
                };
                let yarn = if let Some(yarn) = self.yarn {
                    Some(Version::parse(&yarn).unknown()?)
                } else {
                    None
                };

                Some(PlatformSpec {
                    node_runtime,
                    npm,
                    yarn,
                })
            }
            None => None,
        })
    }

    /// Deserialize the input JSON String into a Platform
    pub fn from_json(src: String) -> Fallible<Self> {
        if src.is_empty() {
            serde_json::de::from_str("{}").unknown()
        } else {
            serde_json::de::from_str(&src).unknown()
        }
    }

    /// Serialize the Platform to a JSON String
    pub fn to_json(self) -> Fallible<String> {
        serde_json::to_string_pretty(&self).unknown()
    }
}

impl PlatformSpec {
    pub fn to_serial(&self) -> Platform {
        Platform {
            node: Some(NodeVersion {
                runtime: self.node_runtime.to_string(),
                npm: self.npm.as_ref().map(|npm| npm.to_string()),
            }),
            yarn: self.yarn.as_ref().map(|yarn| yarn.to_string()),
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use platform;
    use semver;

    // NOTE: serde_json is required with the "preserve_order" feature in Cargo.toml,
    // so these tests will serialized/deserialize in a predictable order

    const BASIC_JSON_STR: &'static str = r#"{
  "node": {
    "runtime": "4.5.6",
    "npm": "7.8.9"
  },
  "yarn": "1.2.3"
}"#;

    #[test]
    fn test_from_json() {
        let json_str = BASIC_JSON_STR.to_string();
        let platform = Platform::from_json(json_str).expect("could not parse JSON string");
        let expected_platform = Platform {
            yarn: Some("1.2.3".to_string()),
            node: Some(NodeVersion {
                runtime: "4.5.6".to_string(),
                npm: Some("7.8.9".to_string()),
            }),
        };
        assert_eq!(platform, expected_platform);
    }

    #[test]
    fn test_from_json_empty_string() {
        let json_str = "".to_string();
        let platform = Platform::from_json(json_str).expect("could not parse JSON string");
        let expected_platform = Platform {
            node: None,
            yarn: None,
        };
        assert_eq!(platform, expected_platform);
    }

    #[test]
    fn test_to_json() {
        let platform = platform::PlatformSpec {
            yarn: Some(semver::Version::parse("1.2.3").expect("could not parse semver version")),
            node_runtime: semver::Version::parse("4.5.6").expect("could not parse semver version"),
            npm: Some(semver::Version::parse("7.8.9").expect("could not parse semver version")),
        };
        let json_str = platform
            .to_serial()
            .to_json()
            .expect("could not serialize platform to JSON");
        let expected_json_str = BASIC_JSON_STR.to_string();
        assert_eq!(json_str, expected_json_str);
    }
}
