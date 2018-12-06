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

impl Toolchain {
    pub fn current() -> Fallible<Toolchain> {
        let path = user_platform_file()?;
        let src = touch(&path)?.read_into_string().unknown()?;
        let serial: serial::Platform = toml::from_str(&src).unknown()?;
        Ok(Toolchain {
            platform: serial.into_image()?
        })
    }

    pub fn get_active_node(&self) -> Option<NodeVersion> {
        self.platform.as_ref().map(|ref p| { p.node.clone() })
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
        self.platform.as_ref().and_then(|ref platform| { platform.yarn.clone() })
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
                let src = toml::to_string_pretty(&platform.to_serial()).unwrap();
                file.write_all(src.as_bytes()).unknown()?;
            }
            &None => {
                file.write_all(b"[platform]\n").unknown()?;
            }
        }
        Ok(())
    }
}
