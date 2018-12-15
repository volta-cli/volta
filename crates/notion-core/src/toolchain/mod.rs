use std::fs::File;
use std::io::Write;

use readext::ReadExt;
use semver::Version;
use toml;

use distro::node::NodeVersion;
use fs::touch;
use image::Image;
use path::user_platform_file;

use notion_fail::{Fallible, ResultExt};

pub(crate) mod serial;

pub struct Toolchain {
    platform: Option<Image>,
}

fn platform_from_toml_str(src: String) -> Fallible<serial::Platform> {
    toml::from_str(&src).unknown()
}

fn platform_to_toml_str(platform: &Image) -> Fallible<String> {
    toml::to_string_pretty(&platform.to_serial()).unknown()
}

impl Toolchain {
    pub fn current() -> Fallible<Toolchain> {
        let path = user_platform_file()?;
        let src = touch(&path)?.read_into_string().unknown()?;
        Ok(Toolchain {
            platform: platform_from_toml_str(src)?.into_image()?,
        })
    }

    pub fn get_active_node(&self) -> Option<NodeVersion> {
        self.platform.as_ref().map(|ref p| p.node.clone())
    }

    pub fn set_active_node(&mut self, version: NodeVersion) -> Fallible<()> {
        let mut dirty = false;

        if let Some(ref mut platform) = self.platform {
            if platform.node != version {
                platform.node = version;
                dirty = true;
            }
        } else {
            self.platform = Some(Image {
                node: version,
                yarn: None,
            });
            dirty = true;
        }

        if dirty {
            self.save()?;
        }

        Ok(())
    }

    pub fn get_active_yarn(&self) -> Option<Version> {
        self.platform
            .as_ref()
            .and_then(|ref platform| platform.yarn.clone())
    }

    pub fn set_active_yarn(&mut self, version: Version) -> Fallible<()> {
        let mut dirty = false;

        if let &mut Some(ref mut platform) = &mut self.platform {
            if platform.yarn != Some(version.clone()) {
                platform.yarn = Some(version);
                dirty = true;
            }
        }

        if dirty {
            self.save()?;
        }

        Ok(())
    }

    pub fn save(&self) -> Fallible<()> {
        let path = user_platform_file()?;
        let mut file = File::create(&path).unknown()?;
        match &self.platform {
            &Some(ref platform) => {
                let src = platform_to_toml_str(platform)?;
                file.write_all(src.as_bytes()).unknown()?;
            }
            &None => {
                file.write_all(b"[platform]\n").unknown()?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {

    use self::serial;
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
    fn test_platform_from_toml_str() {
        let toml_str = BASIC_TOML_STR.to_string();
        let platform = platform_from_toml_str(toml_str).expect("could not parse TOML string");
        let expected_platform = serial::Platform {
            yarn: Some("1.2.3".to_string()),
            node: Some(serial::NodeVersion {
                runtime: "4.5.6".to_string(),
                npm: "7.8.9".to_string(),
            }),
        };
        assert_eq!(platform, expected_platform);
    }

    #[test]
    fn test_platform_to_toml_str() {
        let platform = image::Image {
            yarn: Some(semver::Version::parse("1.2.3").expect("could not parse semver version")),
            node: distro::node::NodeVersion {
                runtime: semver::Version::parse("4.5.6").expect("could not parse semver version"),
                npm: semver::Version::parse("7.8.9").expect("could not parse semver version"),
            },
        };
        let toml_str =
            platform_to_toml_str(&platform).expect("could not serialize platform to TOML");
        let expected_toml_str = BASIC_TOML_STR.to_string();
        assert_eq!(toml_str, expected_toml_str);
    }
}
