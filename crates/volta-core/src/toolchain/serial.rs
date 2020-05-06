use crate::error::{Context, ErrorKind, Fallible};
use crate::platform::PlatformSpec;
use crate::version::{option_version_serde, version_serde};
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NodeVersion {
    #[serde(with = "version_serde")]
    pub runtime: Version,
    #[serde(with = "option_version_serde")]
    pub npm: Option<Version>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Platform {
    #[serde(default)]
    pub node: Option<NodeVersion>,
    #[serde(default)]
    #[serde(with = "option_version_serde")]
    pub yarn: Option<Version>,
}

impl Platform {
    pub fn of(source: &PlatformSpec) -> Self {
        Platform {
            node: Some(NodeVersion {
                runtime: source.node.clone(),
                npm: source.npm.clone(),
            }),
            yarn: source.yarn.clone(),
        }
    }

    pub fn into_platform(self) -> Option<PlatformSpec> {
        let yarn = self.yarn;
        self.node.map(|node_version| PlatformSpec {
            node: node_version.runtime,
            npm: node_version.npm,
            yarn,
        })
    }

    /// Deserialize the input JSON String into a Platform
    pub fn from_json(src: String) -> Fallible<Self> {
        let result = if src.is_empty() {
            serde_json::de::from_str("{}")
        } else {
            serde_json::de::from_str(&src)
        };

        result.with_context(|| ErrorKind::ParsePlatformError)
    }

    /// Serialize the Platform to a JSON String
    pub fn into_json(self) -> Fallible<String> {
        serde_json::to_string_pretty(&self).with_context(|| ErrorKind::StringifyPlatformError)
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::platform;
    use semver::Version;

    // NOTE: serde_json is required with the "preserve_order" feature in Cargo.toml,
    // so these tests will serialized/deserialize in a predictable order

    const BASIC_JSON_STR: &str = r#"{
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
            yarn: Some(Version::parse("1.2.3").expect("could not parse version")),
            node: Some(NodeVersion {
                runtime: Version::parse("4.5.6").expect("could not parse version"),
                npm: Some(Version::parse("7.8.9").expect("could not parse version")),
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
    fn test_into_json() {
        let platform_spec = platform::PlatformSpec {
            yarn: Some(Version::parse("1.2.3").expect("could not parse version")),
            node: Version::parse("4.5.6").expect("could not parse version"),
            npm: Some(Version::parse("7.8.9").expect("could not parse version")),
        };
        let json_str = Platform::of(&platform_spec)
            .into_json()
            .expect("could not serialize platform to JSON");
        let expected_json_str = BASIC_JSON_STR.to_string();
        assert_eq!(json_str, expected_json_str);
    }
}
