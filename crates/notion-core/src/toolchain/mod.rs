use std::fs::File;
use std::io::Write;

use readext::ReadExt;
use semver::Version;

use distro::node::NodeVersion;
use fs::touch;
use path::user_platform_file;
use platform::PlatformSpec;

use notion_fail::{Fallible, ResultExt};

pub(crate) mod serial;

pub struct Toolchain {
    platform: Option<PlatformSpec>,
}

impl Toolchain {
    pub fn current() -> Fallible<Toolchain> {
        let path = user_platform_file()?;
        let src = touch(&path)?.read_into_string().unknown()?;
        Ok(Toolchain {
            platform: serial::Platform::from_json(src)?.into_image()?,
        })
    }

    pub fn platform_ref(&self) -> Option<&PlatformSpec> {
        self.platform.as_ref()
    }

    pub fn set_active_node(&mut self, version: NodeVersion) -> Fallible<()> {
        let mut dirty = false;

        if let Some(ref mut platform) = self.platform {
            if platform.node_runtime != version.runtime {
                platform.node_runtime = version.runtime;
                dirty = true;
            }

            if platform.npm != Some(version.npm.clone()) {
                platform.npm = Some(version.npm);
                dirty = true;
            }
        } else {
            self.platform = Some(PlatformSpec {
                node_runtime: version.runtime,
                npm: Some(version.npm),
                yarn: None,
            });
            dirty = true;
        }

        if dirty {
            self.save()?;
        }

        Ok(())
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
                let src = platform.to_serial().to_json()?;
                file.write_all(src.as_bytes()).unknown()?;
            }
            &None => {
                file.write_all(b"[platform]\n").unknown()?;
            }
        }
        Ok(())
    }
}

