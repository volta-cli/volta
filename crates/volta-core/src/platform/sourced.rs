use std::ffi::OsString;
use std::rc::Rc;

use super::{Image, PlatformSpec};
use crate::session::Session;
use crate::tool::NodeVersion;
use semver::Version;
use volta_fail::Fallible;

pub enum Source {
    /// Represents a Platform that came from the user default
    Default,

    /// Represents a Platform that came from a project manifest
    Project,

    /// Represents a Platform that is the result of merging the Default and Project platforms
    ProjectNodeDefaultYarn,
}

pub struct SourcedPlatformSpec {
    platform: Rc<PlatformSpec>,
    source: Source,
}

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

    pub fn default(platform: Rc<PlatformSpec>) -> Self {
        SourcedPlatformSpec {
            platform,
            source: Source::Default,
        }
    }

    pub fn merged(platform: Rc<PlatformSpec>) -> Self {
        SourcedPlatformSpec {
            platform,
            source: Source::ProjectNodeDefaultYarn,
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
