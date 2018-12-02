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

    pub fn get_installed_node(&self) -> Option<NodeVersion> {
        self.platform.as_ref().map(|ref platform| {
            NodeVersion {
                runtime: platform.node.clone(),
                npm: platform.npm.clone()
            }
        })
    }

    pub fn set_installed_node(&mut self, version: NodeVersion) -> Fallible<()> {
        let mut dirty = false;

        if let &mut Some(ref mut platform) = &mut self.platform {
            if (platform.node != version.runtime) || (platform.npm != version.npm) {
                let node_str = version.runtime.to_string();
                platform.node = version.runtime;
                platform.node_str = node_str;

                let npm_str = version.npm.to_string();
                platform.npm = version.npm;
                platform.npm_str = npm_str;
                dirty = true;
            }
        }

        if dirty {
            self.save()?;
        }

        Ok(())
    }

    pub fn get_installed_yarn(&self) -> Option<Version> {
        self.platform.as_ref().and_then(|ref platform| { platform.yarn.clone() })
    }

    pub fn set_installed_yarn(&mut self, version: Version) -> Fallible<()> {
        let mut dirty = false;

        if let &mut Some(ref mut platform) = &mut self.platform {
            if platform.yarn != Some(version.clone()) {
                let yarn_str = version.to_string();
                platform.yarn = Some(version);
                platform.yarn_str = Some(yarn_str);
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
