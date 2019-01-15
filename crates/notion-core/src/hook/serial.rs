use super::tool;
use std::marker::PhantomData;

use distro::node::NodeDistro;
use distro::yarn::YarnDistro;
use distro::Distro;
use notion_fail::{FailExt, Fallible};

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

#[derive(Fail, Debug)]
#[fail(display = "Hook contains more than one of 'prefix', 'template', or 'bin' fields")]
struct MultipleFieldsSpecified;

#[derive(Fail, Debug)]
#[fail(display = "Hook must contain either a 'prefix', 'template', or 'bin' field")]
struct NoFieldSpecified;

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
            } => Err(NoFieldSpecified.unknown()),
            _ => Err(MultipleFieldsSpecified.unknown()),
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

#[derive(Fail, Debug)]
#[fail(display = "Hook contains both 'url' and 'bin' fields")]
struct BothUrlAndBin;

#[derive(Fail, Debug)]
#[fail(display = "Hook must contain either a 'url' or 'bin' field")]
struct NeitherUrlNorBin;

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
            } => Err(NeitherUrlNorBin.unknown()),
            _ => Err(BothUrlAndBin.unknown()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Hooks {
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

impl Hooks {
    pub fn into_hooks(self) -> Fallible<super::Hooks> {
        Ok(super::Hooks {
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
    pub fn into_tool_hooks(self) -> Fallible<super::ToolHooks<D>> {
        Ok(super::ToolHooks {
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
