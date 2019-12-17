use std::env::JoinPathsError;
use std::fmt;
use std::rc::Rc;

use crate::error::ErrorDetails;
use crate::session::Session;
use crate::tool::{load_default_npm_version, Node, Yarn};
use semver::Version;
use volta_fail::Fallible;

mod image;
mod system;
// Note: The tests get their own module because we need them to run as a single unit to prevent
// clobbering environment variable changes
#[cfg(test)]
mod test;

pub use image::Image;
pub use system::System;

/// The source with which a version is associated
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    /// Represents a version from the user default platform
    Default,

    /// Represents a version from a project manifest
    Project,

    /// Represents a version from a pinned Binary platform
    Binary,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Default => write!(f, "default"),
            Source::Project => write!(f, "project"),
            Source::Binary => write!(f, "binary"),
        }
    }
}

pub struct Sourced<T> {
    pub value: T,
    pub source: Source,
}

impl<T> Sourced<T> {
    pub fn with_default(value: T) -> Self {
        Sourced {
            value,
            source: Source::Default,
        }
    }

    pub fn with_project(value: T) -> Self {
        Sourced {
            value,
            source: Source::Project,
        }
    }

    pub fn with_binary(value: T) -> Self {
        Sourced {
            value,
            source: Source::Binary,
        }
    }
}

impl<T> Sourced<T> {
    pub fn as_ref(&self) -> Sourced<&T> {
        Sourced {
            value: &self.value,
            source: self.source,
        }
    }
}

impl<'a, T> Sourced<&'a T>
where
    T: Clone,
{
    pub fn cloned(self) -> Sourced<T> {
        Sourced {
            value: self.value.clone(),
            source: self.source,
        }
    }
}

pub trait PlatformSpec {
    fn node(&self) -> Sourced<&Version>;
    fn npm(&self) -> Option<Sourced<&Version>>;
    fn yarn(&self) -> Option<Sourced<&Version>>;

    fn checkout(&self, session: &mut Session) -> Fallible<Image> {
        let node = self.node();
        let yarn = self.yarn();
        ensure_node(node.value, session)?;

        if let Some(Sourced { value: version, .. }) = yarn {
            ensure_yarn(version, session)?;
        }

        let npm = match self.npm() {
            Some(n) => n.cloned(),
            None => Sourced {
                value: load_default_npm_version(&node.value)?,
                source: node.source,
            },
        };

        Ok(Image {
            node: node.cloned(),
            npm,
            yarn: yarn.map(|y| y.cloned()),
        })
    }
}

pub struct DefaultPlatformSpec {
    pub node: Version,
    pub npm: Option<Version>,
    pub yarn: Option<Version>,
}

impl PlatformSpec for DefaultPlatformSpec {
    fn node(&self) -> Sourced<&Version> {
        Sourced::with_default(&self.node)
    }

    fn npm(&self) -> Option<Sourced<&Version>> {
        self.npm.as_ref().map(Sourced::with_default)
    }

    fn yarn(&self) -> Option<Sourced<&Version>> {
        self.yarn.as_ref().map(Sourced::with_default)
    }
}

pub struct ProjectPlatformSpec {
    pub node: Version,
    pub npm: Option<Version>,
    pub yarn: Option<Version>,
}

impl ProjectPlatformSpec {
    pub fn merge(
        self: Rc<ProjectPlatformSpec>,
        default: Rc<DefaultPlatformSpec>,
    ) -> Rc<MergedPlatformSpec> {
        Rc::new(MergedPlatformSpec {
            project: self,
            default,
        })
    }
}

impl PlatformSpec for ProjectPlatformSpec {
    fn node(&self) -> Sourced<&Version> {
        Sourced::with_project(&self.node)
    }

    fn npm(&self) -> Option<Sourced<&Version>> {
        self.npm.as_ref().map(Sourced::with_project)
    }

    fn yarn(&self) -> Option<Sourced<&Version>> {
        self.yarn.as_ref().map(Sourced::with_project)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryPlatformSpec {
    pub node: Version,
    pub npm: Option<Version>,
    pub yarn: Option<Version>,
}

impl PlatformSpec for BinaryPlatformSpec {
    fn node(&self) -> Sourced<&Version> {
        Sourced::with_binary(&self.node)
    }

    fn npm(&self) -> Option<Sourced<&Version>> {
        self.npm.as_ref().map(Sourced::with_binary)
    }

    fn yarn(&self) -> Option<Sourced<&Version>> {
        self.yarn.as_ref().map(Sourced::with_binary)
    }
}

pub struct MergedPlatformSpec {
    project: Rc<ProjectPlatformSpec>,
    default: Rc<DefaultPlatformSpec>,
}

impl PlatformSpec for MergedPlatformSpec {
    fn node(&self) -> Sourced<&Version> {
        self.project.node()
    }

    fn npm(&self) -> Option<Sourced<&Version>> {
        self.project.npm().or_else(|| self.default.npm())
    }

    fn yarn(&self) -> Option<Sourced<&Version>> {
        self.project.yarn().or_else(|| self.default.yarn())
    }
}

/// Ensures that a specific Node version has been fetched and unpacked
fn ensure_node(version: &Version, session: &mut Session) -> Fallible<()> {
    let inventory = session.inventory()?;

    if !inventory.node.versions.contains(version) {
        Node::new(version.clone()).fetch_internal(session)?;
    }

    Ok(())
}

/// Ensures that a specific Yarn version has been fetched and unpacked
fn ensure_yarn(version: &Version, session: &mut Session) -> Fallible<()> {
    let inventory = session.inventory()?;

    if !inventory.yarn.versions.contains(version) {
        Yarn::new(version.clone()).fetch_internal(session)?;
    }

    Ok(())
}

fn build_path_error(_err: &JoinPathsError) -> ErrorDetails {
    ErrorDetails::BuildPathError
}
