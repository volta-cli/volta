use std::fs::write;

use crate::error::ErrorDetails;
use crate::fs::touch;
use crate::layout::volta_home;
use crate::platform::PlatformSpec;
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
    platform: Option<PlatformSpec>,
}

impl Toolchain {
    fn current() -> Fallible<Toolchain> {
        let path = volta_home()?.default_platform_file();
        let src = touch(&path)
            .and_then(|mut file| file.read_into_string())
            .with_context(|_| ErrorDetails::ReadPlatformError {
                file: path.to_owned(),
            })?;

        let platform = serial::Platform::from_json(src)?.into_platform();
        if platform.is_some() {
            debug!("Found default configuration at '{}'", path.display());
        }
        Ok(Toolchain { platform })
    }

    pub fn platform(&self) -> Option<&PlatformSpec> {
        self.platform.as_ref()
    }

    /// Set the active Node version in the default platform file.
    pub fn set_active_node(&mut self, node_version: &Version) -> Fallible<()> {
        let mut dirty = false;

        match self.platform.as_mut() {
            Some(platform) => {
                if platform.node != *node_version {
                    platform.node = node_version.clone();
                    dirty = true;
                }
            }
            None => {
                self.platform = Some(PlatformSpec {
                    node: node_version.clone(),
                    npm: None,
                    yarn: None,
                });
                dirty = true;
            }
        }

        if dirty {
            self.save()?;
        }

        Ok(())
    }

    /// Set the active Yarn version in the default platform file.
    pub fn set_active_yarn(&mut self, yarn: Option<Version>) -> Fallible<()> {
        if let Some(platform) = self.platform.as_mut() {
            if platform.yarn != yarn {
                platform.yarn = yarn;
                self.save()?;
            }
        }

        Ok(())
    }

    /// Set the active Npm version in the default platform file.
    pub fn set_active_npm(&mut self, npm: Option<Version>) -> Fallible<()> {
        if let Some(platform) = self.platform.as_mut() {
            if platform.npm != npm {
                platform.npm = npm;
                self.save()?;
            }
        }

        Ok(())
    }

    pub fn save(&self) -> Fallible<()> {
        let path = volta_home()?.default_platform_file();
        let result = match &self.platform {
            Some(platform) => {
                let src = serial::Platform::of(platform).into_json()?;
                write(&path, src)
            }
            None => write(&path, "{}"),
        };
        result.with_context(|_| ErrorDetails::WritePlatformError {
            file: path.to_owned(),
        })
    }
}
