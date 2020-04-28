use std::env::JoinPathsError;
use std::fmt;

use crate::error::ErrorDetails;
use crate::session::Session;
use crate::tool::{Node, Yarn};
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

impl<T> Clone for Sourced<T>
where
    T: Clone,
{
    fn clone(&self) -> Sourced<T> {
        Sourced {
            value: self.value.clone(),
            source: self.source,
        }
    }
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
/// Represents the specification of a single Platform, regardless of the source
pub struct PlatformSpec {
    pub node: Version,
    pub npm: Option<Version>,
    pub yarn: Option<Version>,
}

impl PlatformSpec {
    /// Convert this PlatformSpec into a Platform with all sources set to `Default`
    pub fn as_default(&self) -> Platform {
        Platform {
            node: Sourced::with_default(self.node.clone()),
            npm: self.npm.clone().map(Sourced::with_default),
            yarn: self.yarn.clone().map(Sourced::with_default),
        }
    }

    /// Convert this PlatformSpec into a Platform with all sources set to `Project`
    pub fn as_project(&self) -> Platform {
        Platform {
            node: Sourced::with_project(self.node.clone()),
            npm: self.npm.clone().map(Sourced::with_project),
            yarn: self.yarn.clone().map(Sourced::with_project),
        }
    }

    /// Convert this PlatformSpec into a Platform with all sources set to `Binary`
    pub fn as_binary(&self) -> Platform {
        Platform {
            node: Sourced::with_binary(self.node.clone()),
            npm: self.npm.clone().map(Sourced::with_binary),
            yarn: self.yarn.clone().map(Sourced::with_binary),
        }
    }
}

/// Represents a real Platform, with Versions pulled from one or more `PlatformSpec`s
pub struct Platform {
    pub node: Sourced<Version>,
    pub npm: Option<Sourced<Version>>,
    pub yarn: Option<Sourced<Version>>,
}

impl Platform {
    /// Returns the user's currently active platform, if any
    ///
    /// Active platform is determined by first looking at the Project Platform
    ///
    /// - If it exists and has a Yarn version, then we use the project platform
    /// - If it exists but doesn't have a Yarn version, then we merge the two,
    ///   pulling Yarn from the user default platform, if available
    /// - If there is no Project platform, then we use the user Default Platform
    pub fn current(session: &mut Session) -> Fallible<Option<Self>> {
        match session.project_platform()? {
            Some(platform) => {
                if platform.yarn.is_none() {
                    if let Some(default) = session.default_platform()? {
                        let npm = platform
                            .npm
                            .clone()
                            .map(Sourced::with_project)
                            .or_else(|| default.npm.clone().map(Sourced::with_default));
                        let yarn = platform
                            .yarn
                            .clone()
                            .map(Sourced::with_project)
                            .or_else(|| default.yarn.clone().map(Sourced::with_default));

                        return Ok(Some(Platform {
                            node: Sourced::with_project(platform.node.clone()),
                            npm,
                            yarn,
                        }));
                    }
                }
                Ok(Some(platform.as_project()))
            }
            None => match session.default_platform()? {
                Some(platform) => Ok(Some(platform.as_default())),
                None => Ok(None),
            },
        }
    }

    /// Check out a `Platform` into a fully-realized `Image`
    ///
    /// This will ensure that all necessary tools are fetched and available for execution
    pub fn checkout(self, session: &mut Session) -> Fallible<Image> {
        Node::new(self.node.value.clone()).ensure_fetched(session)?;

        if let Some(Sourced { value: version, .. }) = &self.yarn {
            Yarn::new(version.clone()).ensure_fetched(session)?;
        }

        Ok(Image {
            node: self.node,
            npm: self.npm,
            yarn: self.yarn,
        })
    }
}

fn build_path_error(_err: &JoinPathsError) -> ErrorDetails {
    ErrorDetails::BuildPathError
}
