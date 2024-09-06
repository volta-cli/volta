use std::marker::PhantomData;
use std::path::Path;

use super::tool;
use super::RegistryFormat;
use crate::error::{ErrorKind, Fallible, VoltaError};
use crate::tool::{Node, Npm, Pnpm, Tool};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RawResolveHook {
    prefix: Option<String>,
    template: Option<String>,
    bin: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RawIndexHook {
    prefix: Option<String>,
    template: Option<String>,
    bin: Option<String>,
    format: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RawPublishHook {
    url: Option<String>,
    bin: Option<String>,
}

impl RawResolveHook {
    fn into_hook<H, P, T, B>(self, to_prefix: P, to_template: T, to_bin: B) -> Fallible<H>
    where
        P: FnOnce(String) -> H,
        T: FnOnce(String) -> H,
        B: FnOnce(String) -> H,
    {
        match self {
            RawResolveHook {
                prefix: Some(prefix),
                template: None,
                bin: None,
            } => Ok(to_prefix(prefix)),
            RawResolveHook {
                prefix: None,
                template: Some(template),
                bin: None,
            } => Ok(to_template(template)),
            RawResolveHook {
                prefix: None,
                template: None,
                bin: Some(bin),
            } => Ok(to_bin(bin)),
            RawResolveHook {
                prefix: None,
                template: None,
                bin: None,
            } => Err(ErrorKind::HookNoFieldsSpecified.into()),
            _ => Err(ErrorKind::HookMultipleFieldsSpecified.into()),
        }
    }

    pub fn into_distro_hook(self, base_dir: &Path) -> Fallible<tool::DistroHook> {
        self.into_hook(
            tool::DistroHook::Prefix,
            tool::DistroHook::Template,
            |bin| tool::DistroHook::Bin {
                bin,
                base_path: base_dir.to_owned(),
            },
        )
    }

    pub fn into_metadata_hook(self, base_dir: &Path) -> Fallible<tool::MetadataHook> {
        self.into_hook(
            tool::MetadataHook::Prefix,
            tool::MetadataHook::Template,
            |bin| tool::MetadataHook::Bin {
                bin,
                base_path: base_dir.to_owned(),
            },
        )
    }
}

impl RawIndexHook {
    pub fn into_index_hook(self, base_dir: &Path) -> Fallible<tool::YarnIndexHook> {
        // use user-specified format, or default to Github (legacy)
        let format = match self.format {
            Some(format_str) => RegistryFormat::from_str(&format_str)?,
            None => RegistryFormat::Github,
        };
        Ok(tool::YarnIndexHook {
            format,
            metadata: RawResolveHook {
                prefix: self.prefix,
                template: self.template,
                bin: self.bin,
            }
            .into_metadata_hook(base_dir)?,
        })
    }
}

impl TryFrom<RawPublishHook> for super::Publish {
    type Error = VoltaError;

    fn try_from(raw: RawPublishHook) -> Fallible<super::Publish> {
        match raw {
            RawPublishHook {
                url: Some(url),
                bin: None,
            } => Ok(super::Publish::Url(url)),
            RawPublishHook {
                url: None,
                bin: Some(bin),
            } => Ok(super::Publish::Bin(bin)),
            RawPublishHook {
                url: None,
                bin: None,
            } => Err(ErrorKind::PublishHookNeitherUrlNorBin.into()),
            _ => Err(ErrorKind::PublishHookBothUrlAndBin.into()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RawHookConfig {
    pub node: Option<RawToolHooks<Node>>,
    pub npm: Option<RawToolHooks<Npm>>,
    pub pnpm: Option<RawToolHooks<Pnpm>>,
    pub yarn: Option<RawYarnHooks>,
    pub events: Option<RawEventHooks>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "events")]
pub struct RawEventHooks {
    pub publish: Option<RawPublishHook>,
}

impl TryFrom<RawEventHooks> for super::EventHooks {
    type Error = VoltaError;

    fn try_from(raw: RawEventHooks) -> Fallible<super::EventHooks> {
        let publish = raw.publish.map(|p| p.try_into()).transpose()?;

        Ok(super::EventHooks { publish })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "tool")]
pub struct RawToolHooks<T: Tool> {
    pub distro: Option<RawResolveHook>,
    pub latest: Option<RawResolveHook>,
    pub index: Option<RawResolveHook>,

    #[serde(skip)]
    phantom: PhantomData<T>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "yarn")]
pub struct RawYarnHooks {
    pub distro: Option<RawResolveHook>,
    pub latest: Option<RawResolveHook>,
    pub index: Option<RawIndexHook>,
}

impl RawHookConfig {
    pub fn into_hook_config(self, base_dir: &Path) -> Fallible<super::HookConfig> {
        let node = self.node.map(|n| n.into_tool_hooks(base_dir)).transpose()?;
        let npm = self.npm.map(|n| n.into_tool_hooks(base_dir)).transpose()?;
        let pnpm = self.pnpm.map(|p| p.into_tool_hooks(base_dir)).transpose()?;
        let yarn = self.yarn.map(|y| y.into_yarn_hooks(base_dir)).transpose()?;
        let events = self.events.map(|e| e.try_into()).transpose()?;
        Ok(super::HookConfig {
            node,
            npm,
            pnpm,
            yarn,
            events,
        })
    }
}

impl<T: Tool> RawToolHooks<T> {
    pub fn into_tool_hooks(self, base_dir: &Path) -> Fallible<super::ToolHooks<T>> {
        let distro = self
            .distro
            .map(|d| d.into_distro_hook(base_dir))
            .transpose()?;
        let latest = self
            .latest
            .map(|d| d.into_metadata_hook(base_dir))
            .transpose()?;
        let index = self
            .index
            .map(|d| d.into_metadata_hook(base_dir))
            .transpose()?;

        Ok(super::ToolHooks {
            distro,
            latest,
            index,
            phantom: PhantomData,
        })
    }
}

impl RawYarnHooks {
    pub fn into_yarn_hooks(self, base_dir: &Path) -> Fallible<super::YarnHooks> {
        let distro = self
            .distro
            .map(|d| d.into_distro_hook(base_dir))
            .transpose()?;
        let latest = self
            .latest
            .map(|d| d.into_metadata_hook(base_dir))
            .transpose()?;
        let index = self
            .index
            .map(|d| d.into_index_hook(base_dir))
            .transpose()?;

        Ok(super::YarnHooks {
            distro,
            latest,
            index,
        })
    }
}
