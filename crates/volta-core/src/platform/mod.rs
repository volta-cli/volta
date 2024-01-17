use std::env;
use std::fmt;

use crate::error::{ErrorKind, Fallible};
use crate::session::Session;
use crate::tool::{Node, Npm, Pnpm, Yarn};
use crate::VOLTA_FEATURE_PNPM;
use node_semver::Version;

mod image;
mod system;
// Note: The tests get their own module because we need them to run as a single unit to prevent
// clobbering environment variable changes
#[cfg(test)]
mod tests;

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
#[derive(Clone, Default)]
pub enum InheritOption<T> {
    Some(T),
    None,
    #[default]
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

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
#[cfg_attr(test, derive(Debug))]
/// Represents the specification of a single Platform, regardless of the source
pub struct PlatformSpec {
    pub node: Version,
    pub npm: Option<Version>,
    pub pnpm: Option<Version>,
    pub yarn: Option<Version>,
}

impl PlatformSpec {
    /// Convert this PlatformSpec into a Platform with all sources set to `Default`
    pub fn as_default(&self) -> Platform {
        Platform {
            node: Sourced::with_default(self.node.clone()),
            npm: self.npm.clone().map(Sourced::with_default),
            pnpm: self.pnpm.clone().map(Sourced::with_default),
            yarn: self.yarn.clone().map(Sourced::with_default),
        }
    }

    /// Convert this PlatformSpec into a Platform with all sources set to `Project`
    pub fn as_project(&self) -> Platform {
        Platform {
            node: Sourced::with_project(self.node.clone()),
            npm: self.npm.clone().map(Sourced::with_project),
            pnpm: self.pnpm.clone().map(Sourced::with_project),
            yarn: self.yarn.clone().map(Sourced::with_project),
        }
    }

    /// Convert this PlatformSpec into a Platform with all sources set to `Binary`
    pub fn as_binary(&self) -> Platform {
        Platform {
            node: Sourced::with_binary(self.node.clone()),
            npm: self.npm.clone().map(Sourced::with_binary),
            pnpm: self.pnpm.clone().map(Sourced::with_binary),
            yarn: self.yarn.clone().map(Sourced::with_binary),
        }
    }
}

/// Represents a (maybe) platform with values from the command line
#[derive(Clone)]
pub struct CliPlatform {
    pub node: Option<Version>,
    pub npm: InheritOption<Version>,
    pub pnpm: InheritOption<Version>,
    pub yarn: InheritOption<Version>,
}

impl CliPlatform {
    /// Merges the `CliPlatform` with a `Platform`, inheriting from the base where needed
    pub fn merge(self, base: Platform) -> Platform {
        Platform {
            node: self.node.map_or(base.node, Sourced::with_command_line),
            npm: self.npm.map(Sourced::with_command_line).inherit(base.npm),
            pnpm: self.pnpm.map(Sourced::with_command_line).inherit(base.pnpm),
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
                pnpm: base.pnpm.map(Sourced::with_command_line).into(),
                yarn: base.yarn.map(Sourced::with_command_line).into(),
            }),
        }
    }
}

/// Represents a real Platform, with Versions pulled from one or more `PlatformSpec`s
#[derive(Clone)]
pub struct Platform {
    pub node: Sourced<Version>,
    pub npm: Option<Sourced<Version>>,
    pub pnpm: Option<Sourced<Version>>,
    pub yarn: Option<Sourced<Version>>,
}

impl Platform {
    /// Returns the user's currently active platform, if any
    ///
    /// Active platform is determined by first looking at the Project Platform
    ///
    /// - If there is a project platform then we use it
    ///   - If there is no pnpm/Yarn version in the project platform, we pull
    ///     pnpm/Yarn from the default platform if available, and merge the two
    ///     platforms into a final one
    /// - If there is no Project platform, then we use the user Default Platform
    pub fn current(session: &mut Session) -> Fallible<Option<Self>> {
        if let Some(mut platform) = session.project_platform()?.map(PlatformSpec::as_project) {
            if platform.pnpm.is_none() {
                platform.pnpm = session
                    .default_platform()?
                    .and_then(|default_platform| default_platform.pnpm.clone())
                    .map(Sourced::with_default);
            }

            if platform.yarn.is_none() {
                platform.yarn = session
                    .default_platform()?
                    .and_then(|default_platform| default_platform.yarn.clone())
                    .map(Sourced::with_default);
            }

            Ok(Some(platform))
        } else {
            Ok(session.default_platform()?.map(PlatformSpec::as_default))
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

        // Only force download of the pnpm version if the pnpm feature flag is set. If it isn't,
        // then we won't be using the `Pnpm` tool to execute (we will be relying on the global
        // package logic), so fetching the Pnpm version would only be redundant work.
        if env::var_os(VOLTA_FEATURE_PNPM).is_some() {
            if let Some(Sourced { value: version, .. }) = &self.pnpm {
                Pnpm::new(version.clone()).ensure_fetched(session)?;
            }
        }

        if let Some(Sourced { value: version, .. }) = &self.yarn {
            Yarn::new(version.clone()).ensure_fetched(session)?;
        }

        Ok(Image {
            node: self.node,
            npm: self.npm,
            pnpm: self.pnpm,
            yarn: self.yarn,
        })
    }
}

fn build_path_error() -> ErrorKind {
    ErrorKind::BuildPathError
}
