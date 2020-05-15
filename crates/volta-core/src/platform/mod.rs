use std::fmt;

use crate::error::{ErrorKind, Fallible};
use crate::session::Session;
use crate::tool::{Node, Npm, Yarn};
use semver::Version;

mod image;
mod system;
// Note: The tests get their own module because we need them to run as a single unit to prevent
// clobbering environment variable changes
#[cfg(test)]
mod test;

pub use image::Image;
pub use system::System;

/// The source with which a version is associated
#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Eq, PartialEq, Debug))]
pub enum Source {
    /// Represents a version from the user default platform
    Default,

    /// Represents a version from a project manifest
    Project,

    /// Represents a version from a pinned Binary platform
    Binary,

    /// Represents a version from the command line (via `volta run`)
    CommandLine,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Default => write!(f, "default"),
            Source::Project => write!(f, "project"),
            Source::Binary => write!(f, "binary"),
            Source::CommandLine => write!(f, "command-line"),
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

    pub fn with_command_line(value: T) -> Self {
        Sourced {
            value,
            source: Source::CommandLine,
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

/// Represents 3 possible states: Having a value, not having a value, and inheriting a value
#[cfg_attr(test, derive(Eq, PartialEq, Debug))]
pub enum InheritOption<T> {
    Some(T),
    None,
    Inherit,
}

impl<T> InheritOption<T> {
    /// Applies a function to the contained value (if any)
    pub fn map<U, F>(self, f: F) -> InheritOption<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            InheritOption::Some(value) => InheritOption::Some(f(value)),
            InheritOption::None => InheritOption::None,
            InheritOption::Inherit => InheritOption::Inherit,
        }
    }

    /// Converts the `InheritOption` into a regular `Option` by inheriting from the provided value if needed
    pub fn inherit(self, other: Option<T>) -> Option<T> {
        match self {
            InheritOption::Some(value) => Some(value),
            InheritOption::None => None,
            InheritOption::Inherit => other,
        }
    }
}

impl<T> From<InheritOption<T>> for Option<T> {
    fn from(base: InheritOption<T>) -> Option<T> {
        base.inherit(None)
    }
}

impl<T> Default for InheritOption<T> {
    fn default() -> Self {
        InheritOption::Inherit
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

/// Represents a (maybe) platform with values from the command line
#[derive(Default)]
pub struct CliPlatform {
    pub node: Option<Version>,
    pub npm: InheritOption<Version>,
    pub yarn: InheritOption<Version>,
}

impl CliPlatform {
    /// Merges the `CliPlatform` with a `Platform`, inheriting from the base where needed
    pub fn merge(self, base: Platform) -> Platform {
        Platform {
            node: self.node.map_or(base.node, Sourced::with_command_line),
            npm: self.npm.map(Sourced::with_command_line).inherit(base.npm),
            yarn: self.yarn.map(Sourced::with_command_line).inherit(base.yarn),
        }
    }
}

impl From<CliPlatform> for Option<Platform> {
    /// Converts the `CliPlatform` into a possible Platform without a base from which to inherit
    fn from(base: CliPlatform) -> Option<Platform> {
        match base.node {
            None => None,
            Some(node) => Some(Platform {
                node: Sourced::with_command_line(node),
                npm: base.npm.map(Sourced::with_command_line).into(),
                yarn: base.yarn.map(Sourced::with_command_line).into(),
            }),
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
                if platform.yarn.is_none() || platform.npm.is_none() {
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

    /// Returns the platform created by merging a `CliPartialPlatform` with the currently active platform
    pub fn with_cli(cli: CliPlatform, session: &mut Session) -> Fallible<Option<Self>> {
        match Self::current(session)? {
            Some(current) => Ok(Some(cli.merge(current))),
            None => Ok(cli.into()),
        }
    }

    /// Check out a `Platform` into a fully-realized `Image`
    ///
    /// This will ensure that all necessary tools are fetched and available for execution
    pub fn checkout(self, session: &mut Session) -> Fallible<Image> {
        Node::new(self.node.value.clone()).ensure_fetched(session)?;

        if let Some(Sourced { value: version, .. }) = &self.npm {
            Npm::new(version.clone()).ensure_fetched(session)?;
        }

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

fn build_path_error() -> ErrorKind {
    ErrorKind::BuildPathError
}
