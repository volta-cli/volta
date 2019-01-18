use std::fs::File;
use std::io::Write;

use readext::ReadExt;

use distro::DistroVersion;
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

    /// Set the active tool versions in the user platform file.
    pub fn set_active(&mut self, distro_version: DistroVersion) -> Fallible<()> {
        let mut dirty = false;

        match distro_version {
            DistroVersion::Node(node, npm) => {
                if let Some(ref mut platform) = self.platform {
                    if platform.node_runtime != node {
                        platform.node_runtime = node;
                        dirty = true;
                    }

                    if platform.npm != Some(npm.clone()) {
                        platform.npm = Some(npm);
                        dirty = true;
                    }
                } else {
                    self.platform = Some(PlatformSpec {
                        node_runtime: node,
                        npm: Some(npm),
                        yarn: None,
                    });
                    dirty = true;
                }
            }
            DistroVersion::Yarn(version) => {
                if let &mut Some(ref mut platform) = &mut self.platform {
                    if platform.yarn != Some(version.clone()) {
                        platform.yarn = Some(version);
                        dirty = true;
                    }
                }
            }
            // ISSUE (#175) When we can `notion install npm` then it can be set in the platform file.
            DistroVersion::Npm(_) => unimplemented!("cannot set npm in platform file"),
            DistroVersion::Package(name, _) => {
                unimplemented!("cannot set {} in platform file", name)
            }
        }

        // both
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
                file.write_all(b"{}").unknown()?;
            }
        }
        Ok(())
    }
}
