use std::ffi::OsString;
use std::rc::Rc;

use super::{Image, PlatformSpec};
use crate::distro::node::NodeVersion;
use crate::session::Session;
use crate::source::Source;
use semver::Version;
use volta_fail::Fallible;

#[derive(Debug)]
pub struct SourcedPlatformSpec {
    platform: Rc<PlatformSpec>,
    source: Source,
}

#[derive(Debug)]
pub struct SourcedImage {
    image: Image,
    source: Source,
}

impl SourcedPlatformSpec {
    pub fn project(platform: Rc<PlatformSpec>) -> Self {
        SourcedPlatformSpec {
            platform,
            source: Source::Project,
        }
    }

    pub fn user(platform: Rc<PlatformSpec>) -> Self {
        SourcedPlatformSpec {
            platform,
            source: Source::User,
        }
    }

    pub fn checkout(self, session: &mut Session) -> Fallible<SourcedImage> {
        let image = self.platform.checkout(session)?;
        Ok(SourcedImage {
            image,
            source: self.source,
        })
    }

    pub fn node(&self) -> &Version {
        &self.platform.node_runtime
    }

    pub fn npm(&self) -> Option<&Version> {
        self.platform.npm.as_ref()
    }

    pub fn yarn(&self) -> Option<&Version> {
        self.platform.yarn.as_ref()
    }

    pub fn source(&self) -> &Source {
        &self.source
    }
}

impl SourcedImage {
    pub fn path(&self) -> Fallible<OsString> {
        self.image.path()
    }

    pub fn node(&self) -> &NodeVersion {
        &self.image.node
    }

    pub fn yarn(&self) -> Option<&Version> {
        self.image.yarn.as_ref()
    }

    pub fn source(&self) -> &Source {
        &self.source
    }
}
