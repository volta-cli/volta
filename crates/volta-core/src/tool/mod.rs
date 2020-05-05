use std::fmt::{self, Display};

use crate::error::ErrorDetails;
use crate::session::Session;
use crate::style::{note_prefix, success_prefix, tool_version};
use crate::version::VersionSpec;
use log::{debug, info};
use volta_fail::Fallible;

pub mod node;
pub mod npm;
pub mod package;
mod serial;
pub mod yarn;

pub use node::{
    load_default_npm_version, Node, NODE_DISTRO_ARCH, NODE_DISTRO_EXTENSION, NODE_DISTRO_OS,
};
pub use npm::{BundledNpm, Npm};
pub use package::{bin_full_path, BinConfig, BinLoader, Package, PackageConfig, PackageDetails};
pub use yarn::Yarn;

#[inline]
fn debug_already_fetched<T: Display + Sized>(tool: T) {
    debug!("{} has already been fetched, skipping download", tool);
}

#[inline]
fn info_installed<T: Display + Sized>(tool: T) {
    info!("{} installed and set {} as default", success_prefix(), tool);
}

#[inline]
fn info_fetched<T: Display + Sized>(tool: T) {
    info!("{} fetched {}", success_prefix(), tool);
}

#[inline]
fn info_pinned<T: Display + Sized>(tool: T) {
    info!("{} pinned {} in package.json", success_prefix(), tool);
}

#[inline]
fn info_project_version<T: Display + Sized>(tool: T) {
    info!(
        "{} you are using {} in the current project",
        note_prefix(),
        tool
    );
}

/// Trait representing all of the actions that can be taken with a tool
pub trait Tool: Display {
    /// Fetch a Tool into the local inventory
    fn fetch(self: Box<Self>, session: &mut Session) -> Fallible<()>;
    /// Install a tool, making it the default so it is available everywhere on the user's machine
    fn install(self: Box<Self>, session: &mut Session) -> Fallible<()>;
    /// Pin a tool in the local project so that it is usable within the project
    fn pin(self: Box<Self>, session: &mut Session) -> Fallible<()>;
}

/// Specification for a tool and its associated version.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Spec {
    Node(VersionSpec),
    Npm(VersionSpec),
    Yarn(VersionSpec),
    Package(String, VersionSpec),
}

impl Spec {
    /// Resolve a tool spec into a fully realized Tool that can be fetched
    pub fn resolve(self, session: &mut Session) -> Fallible<Box<dyn Tool>> {
        match self {
            Spec::Node(version) => {
                let version = node::resolve(version, session)?;
                Ok(Box::new(Node::new(version)))
            }
            Spec::Npm(version) => match npm::resolve(version, session)? {
                Some(version) => Ok(Box::new(Npm::new(version))),
                None => Ok(Box::new(BundledNpm)),
            },
            Spec::Yarn(version) => {
                let version = yarn::resolve(version, session)?;
                Ok(Box::new(Yarn::new(version)))
            }
            Spec::Package(name, version) => {
                let details = package::resolve(&name, version, session)?;
                Ok(Box::new(Package::new(name, details)))
            }
        }
    }

    /// Uninstall a tool, removing it from the local inventory
    ///
    /// This is implemented on Spec, instead of Resolved, because there is currently no need to
    /// resolve the specific version before uninstalling a tool.
    pub fn uninstall(self) -> Fallible<()> {
        match self {
            Spec::Node(_) => Err(ErrorDetails::Unimplemented {
                feature: "Uninstalling node".into(),
            }
            .into()),
            Spec::Npm(_) => Err(ErrorDetails::Unimplemented {
                feature: "Uninstalling npm".into(),
            }
            .into()),
            Spec::Yarn(_) => Err(ErrorDetails::Unimplemented {
                feature: "Uninstalling yarn".into(),
            }
            .into()),
            Spec::Package(name, _) => {
                package::uninstall(&name)?;
                Ok(())
            }
        }
    }
}

impl Display for Spec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Spec::Node(ref version) => tool_version("node", version),
            Spec::Npm(ref version) => tool_version("npm", version),
            Spec::Yarn(ref version) => tool_version("yarn", version),
            Spec::Package(ref name, ref version) => tool_version(name, version),
        };
        f.write_str(&s)
    }
}

fn download_tool_error(
    tool: Spec,
    from_url: impl AsRef<str>,
) -> impl FnOnce(&failure::Error) -> ErrorDetails {
    let from_url = from_url.as_ref().to_string();
    |_| ErrorDetails::DownloadToolNetworkError { tool, from_url }
}

fn registry_fetch_error(
    tool: impl AsRef<str>,
    from_url: impl AsRef<str>,
) -> impl FnOnce(&attohttpc::Error) -> ErrorDetails {
    let tool = tool.as_ref().to_string();
    let from_url = from_url.as_ref().to_string();
    |_| ErrorDetails::RegistryFetchError { tool, from_url }
}
