use std::fs::File;
use std::io::Write;

use readext::ReadExt;
use semver::Version;

use distro::node::NodeVersion;
use distro::DistroVersion;
use fs::touch;
use path::user_platform_file;
use platform::PlatformSpec;
// use tool::Tool;

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

    pub fn get_active_node(&self) -> Option<NodeVersion> {
        self.platform.as_ref().map(|ref p| p.node.clone())
    }

    pub fn get_active_yarn(&self) -> Option<Version> {
        self.platform
            .as_ref()
            .and_then(|ref platform| platform.yarn.clone())
    }

    /// Set the active tool versions in the user platform file.
    pub fn set_active(&mut self, distro_version: DistroVersion) -> Fallible<()> {
        let mut dirty = false;

        match distro_version {
            DistroVersion::Node(node, npm) => {
                let node_version = NodeVersion { runtime: node, npm: npm };
                if let Some(ref mut platform) = self.platform {
                    if platform.node != node_version {
                        platform.node = node_version;
                        dirty = true;
                    }
                } else {
                    self.platform = Some(PlatformSpec {
                        node: node_version,
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
            DistroVersion::Npm(_) => unimplemented!("TODO"),
            DistroVersion::Npx(_) => unimplemented!("TODO"),
            DistroVersion::Package(name, _) => unimplemented!("{} TODO", name),
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
                file.write_all(b"[platform]\n").unknown()?;
            }
        }
        Ok(())
    }
}

