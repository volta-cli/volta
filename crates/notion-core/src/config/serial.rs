use super::super::config;
use std::marker::PhantomData;

use distro::node::NodeDistro;
use distro::yarn::YarnDistro;
use distro::Distro;
use hook::serial::{PublishHook, ToolHook};

use notion_fail::Fallible;

#[derive(Serialize, Deserialize)]
pub struct HookConfig {
    pub node: Option<ToolHooks<NodeDistro>>,
    pub yarn: Option<ToolHooks<YarnDistro>>,
    pub events: Option<EventHooks>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "events")]
pub struct EventHooks {
    pub publish: Option<PublishHook>,
}

impl EventHooks {
    pub fn into_event_hooks(self) -> Fallible<config::EventHooks> {
        Ok(config::EventHooks {
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
    pub distro: Option<ToolHook>,
    pub latest: Option<ToolHook>,
    pub index: Option<ToolHook>,

    #[serde(skip)]
    phantom: PhantomData<I>,
}

impl HookConfig {
    pub fn into_hook_config(self) -> Fallible<config::HookConfig> {
        Ok(config::HookConfig {
            node: if let Some(n) = self.node {
                Some(n.into_tool_hooks()?)
            } else {
                None
            },
            yarn: if let Some(y) = self.yarn {
                Some(y.into_tool_hooks()?)
            } else {
                None
            },
            events: if let Some(e) = self.events {
                Some(e.into_event_hooks()?)
            } else {
                None
            },
        })
    }
}

impl<D: Distro> ToolHooks<D> {
    pub fn into_tool_hooks(self) -> Fallible<config::ToolHooks<D>> {
        Ok(config::ToolHooks {
            distro: if let Some(h) = self.distro {
                Some(h.into_distro_hook()?)
            } else {
                None
            },
            latest: if let Some(h) = self.latest {
                Some(h.into_metadata_hook()?)
            } else {
                None
            },
            index: if let Some(h) = self.index {
                Some(h.into_metadata_hook()?)
            } else {
                None
            },
            phantom: PhantomData,
        })
    }
}
