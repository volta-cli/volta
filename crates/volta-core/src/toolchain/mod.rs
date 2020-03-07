use std::fs::write;
use std::rc::Rc;

use crate::error::ErrorDetails;
use crate::fs::touch;
use crate::layout::volta_home;
use crate::platform::DefaultPlatformSpec;
use lazycell::LazyCell;
use log::debug;
use readext::ReadExt;
use semver::Version;
use volta_fail::{Fallible, ResultExt};

pub mod serial;

/// Lazily loaded toolchain
pub struct LazyToolchain {
    toolchain: LazyCell<Toolchain>,
}

impl LazyToolchain {
    /// Creates a new `LazyToolchain`
    pub fn init() -> Self {
        LazyToolchain {
            toolchain: LazyCell::new(),
        }
    }

    /// Forces loading of the toolchain and returns an immutable reference to it
    pub fn get(&self) -> Fallible<&Toolchain> {
        self.toolchain.try_borrow_with(Toolchain::current)
    }

    /// Forces loading of the toolchain and returns a mutable reference to it
    pub fn get_mut(&mut self) -> Fallible<&mut Toolchain> {
        self.toolchain.try_borrow_mut_with(Toolchain::current)
    }
}

pub struct Toolchain {
    platform: Option<Rc<DefaultPlatformSpec>>,
}

impl Toolchain {
    fn current() -> Fallible<Toolchain> {
        let path = volta_home()?.default_platform_file();
        let src = touch(&path)
            .and_then(|mut file| file.read_into_string())
            .with_context(|_| ErrorDetails::ReadPlatformError {
                file: path.to_owned(),
            })?;

        let platform = serial::Platform::from_json(src)?
            .into_default_platform()
            .map(Rc::new);
        if platform.is_some() {
            debug!("Found default configuration at '{}'", path.display());
        }
        Ok(Toolchain { platform })
    }

    pub fn platform(&self) -> Option<Rc<DefaultPlatformSpec>> {
        self.platform.clone()
    }

    /// Set the active Node version in the default platform file.
    pub fn set_active_node(&mut self, node_version: &Version) -> Fallible<()> {
        let mut dirty = false;

        if let Some(platform) = &self.platform {
            if platform.node != *node_version {
                self.platform = Some(Rc::new(DefaultPlatformSpec {
                    node: node_version.clone(),
                    npm: platform.npm.clone(),
                    yarn: platform.yarn.clone(),
                }));
                dirty = true;
            }
        } else {
            self.platform = Some(Rc::new(DefaultPlatformSpec {
                node: node_version.clone(),
                npm: None,
                yarn: None,
            }));
            dirty = true;
        }

        if dirty {
            self.save()?;
        }

        Ok(())
    }

    /// Set the active Yarn version in the default platform file.
    pub fn set_active_yarn(&mut self, yarn: Option<Version>) -> Fallible<()> {
        if let Some(platform) = &self.platform {
            if platform.yarn != yarn {
                self.platform = Some(Rc::new(DefaultPlatformSpec {
                    node: platform.node.clone(),
                    npm: platform.npm.clone(),
                    yarn,
                }));
                self.save()?;
            }
        }

        Ok(())
    }

    /// Set the active Npm version in the default platform file.
    pub fn set_active_npm(&mut self, npm: Option<Version>) -> Fallible<()> {
        if let Some(platform) = &self.platform {
            if platform.npm != npm {
                self.platform = Some(Rc::new(DefaultPlatformSpec {
                    node: platform.node.clone(),
                    npm,
                    yarn: platform.yarn.clone(),
                }));
                self.save()?;
            }
        }

        Ok(())
    }

    pub fn save(&self) -> Fallible<()> {
        let path = volta_home()?.default_platform_file();
        let result = match &self.platform {
            Some(platform) => {
                let src = platform.to_serial().into_json()?;
                write(&path, src)
            }
            None => write(&path, "{}"),
        };
        result.with_context(|_| ErrorDetails::WritePlatformError {
            file: path.to_owned(),
        })
    }
}
