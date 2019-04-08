use super::tool;
use std::marker::PhantomData;

use crate::distro::node::NodeDistro;
use crate::distro::package::PackageDistro;
use crate::distro::yarn::YarnDistro;
use crate::distro::Distro;
use crate::error::ErrorDetails;
use notion_fail::Fallible;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ResolveHook {
    prefix: Option<String>,
    template: Option<String>,
    bin: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PublishHook {
    url: Option<String>,
    bin: Option<String>,
}

impl ResolveHook {
    fn into_hook<H, P, T, B>(self, to_prefix: P, to_template: T, to_bin: B) -> Fallible<H>
    where
        P: FnOnce(String) -> H,
        T: FnOnce(String) -> H,
        B: FnOnce(String) -> H,
    {
        match self {
            ResolveHook {
                prefix: Some(prefix),
                template: None,
                bin: None,
            } => Ok(to_prefix(prefix)),
            ResolveHook {
                prefix: None,
                template: Some(template),
                bin: None,
            } => Ok(to_template(template)),
            ResolveHook {
                prefix: None,
                template: None,
                bin: Some(bin),
            } => Ok(to_bin(bin)),
            ResolveHook {
                prefix: None,
                template: None,
                bin: None,
            } => Err(ErrorDetails::HookNoFieldsSpecified.into()),
            _ => Err(ErrorDetails::HookMultipleFieldsSpecified.into()),
        }
    }

    pub fn into_distro_hook(self) -> Fallible<tool::DistroHook> {
        self.into_hook(
            tool::DistroHook::Prefix,
            tool::DistroHook::Template,
            tool::DistroHook::Bin,
        )
    }

    pub fn into_metadata_hook(self) -> Fallible<tool::MetadataHook> {
        self.into_hook(
            tool::MetadataHook::Prefix,
            tool::MetadataHook::Template,
            tool::MetadataHook::Bin,
        )
    }
}

impl PublishHook {
    pub fn into_publish(self) -> Fallible<super::Publish> {
        match self {
            PublishHook {
                url: Some(url),
                bin: None,
            } => Ok(super::Publish::Url(url)),
            PublishHook {
                url: None,
                bin: Some(bin),
            } => Ok(super::Publish::Bin(bin)),
            PublishHook {
                url: None,
                bin: None,
            } => Err(ErrorDetails::PublishHookNeitherUrlNorBin.into()),
            _ => Err(ErrorDetails::PublishHookBothUrlAndBin.into()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HookConfig {
    pub node: Option<ToolHooks<NodeDistro>>,
    pub yarn: Option<ToolHooks<YarnDistro>>,
    pub packages: Option<ToolHooks<PackageDistro>>,
    pub events: Option<EventHooks>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "events")]
pub struct EventHooks {
    pub publish: Option<PublishHook>,
}

impl EventHooks {
    pub fn into_event_hooks(self) -> Fallible<super::EventHooks> {
        Ok(super::EventHooks {
            publish: if let Some(p) = self.publish {
                Some(p.into_publish()?)
            } else {
                None
            },
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "tool")]
pub struct ToolHooks<I> {
    pub distro: Option<ResolveHook>,
    pub latest: Option<ResolveHook>,
    pub index: Option<ResolveHook>,

    #[serde(skip)]
    phantom: PhantomData<I>,
}

impl HookConfig {
    pub fn into_hook_config(self) -> Fallible<super::HookConfig> {
        let node = self.node.map(|n| n.into_tool_hooks()).transpose()?;
        let yarn = self.yarn.map(|y| y.into_tool_hooks()).transpose()?;
        let package = self.packages.map(|p| p.into_tool_hooks()).transpose()?;
        let events = self.events.map(|e| e.into_event_hooks()).transpose()?;
        Ok(super::HookConfig {
            node,
            yarn,
            package,
            events,
        })
    }
}

impl<D: Distro> ToolHooks<D> {
    pub fn into_tool_hooks(self) -> Fallible<super::ToolHooks<D>> {
        let distro = self.distro.map(|d| d.into_distro_hook()).transpose()?;
        let latest = self.latest.map(|d| d.into_metadata_hook()).transpose()?;
        let index = self.index.map(|d| d.into_metadata_hook()).transpose()?;

        Ok(super::ToolHooks {
            distro,
            latest,
            index,
            phantom: PhantomData,
        })
    }
}
