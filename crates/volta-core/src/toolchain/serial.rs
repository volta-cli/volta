use crate::error::{Context, ErrorKind, Fallible, VoltaError};
use crate::platform::PlatformSpec;
use crate::version::{option_version_serde, version_serde};
use node_semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct NodeVersion {
    #[serde(with = "version_serde")]
    pub runtime: Version,
    #[serde(with = "option_version_serde")]
    pub npm: Option<Version>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Platform {
    #[serde(default)]
    pub node: Option<NodeVersion>,
    #[serde(default)]
    #[serde(with = "option_version_serde")]
    pub pnpm: Option<Version>,
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
            pnpm: source.pnpm.clone(),
            yarn: source.yarn.clone(),
        }
    }

    /// Serialize the Platform to a JSON String
    pub fn into_json(self) -> Fallible<String> {
        serde_json::to_string_pretty(&self).with_context(|| ErrorKind::StringifyPlatformError)
    }
}

impl TryFrom<String> for Platform {
    type Error = VoltaError;
    fn try_from(src: String) -> Fallible<Self> {
        let result = if src.is_empty() {
            serde_json::de::from_str("{}")
        } else {
            serde_json::de::from_str(&src)
        };

        result.with_context(|| ErrorKind::ParsePlatformError)
    }
}

impl From<Platform> for Option<PlatformSpec> {
    fn from(platform: Platform) -> Option<PlatformSpec> {
        let yarn = platform.yarn;
        let pnpm = platform.pnpm;
        platform.node.map(|node_version| PlatformSpec {
            node: node_version.runtime,
            npm: node_version.npm,
            pnpm,
            yarn,
        })
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::platform;
    use node_semver::Version;

    // NOTE: serde_json is required with the "preserve_order" feature in Cargo.toml,
    // so these tests will serialized/deserialize in a predictable order

    const BASIC_JSON_STR: &str = r#"{
  "node": {
    "runtime": "4.5.6",
    "npm": "7.8.9"
  },
  "pnpm": "3.2.1",
  "yarn": "1.2.3"
}"#;

    #[test]
    fn test_from_json() {
        let json_str = BASIC_JSON_STR.to_string();
        let platform = Platform::try_from(json_str).expect("could not parse JSON string");
        let expected_platform = Platform {
            pnpm: Some(Version::parse("3.2.1").expect("could not parse version")),
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
        let platform = Platform::try_from(json_str).expect("could not parse JSON string");
        let expected_platform = Platform {
            node: None,
            pnpm: None,
            yarn: None,
        };
        assert_eq!(platform, expected_platform);
    }

    #[test]
    fn test_into_json() {
        let platform_spec = platform::PlatformSpec {
            pnpm: Some(Version::parse("3.2.1").expect("could not parse version")),
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
