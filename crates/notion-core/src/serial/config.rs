use super::super::config;
use std::marker::PhantomData;

use super::plugin::Plugin;
use installer::Install;
use installer::node::NodeInstaller;
use installer::yarn::YarnInstaller;

use notion_fail::Fallible;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub node: Option<ToolConfig<NodeInstaller>>,
    pub yarn: Option<ToolConfig<YarnInstaller>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "tool")]
pub struct ToolConfig<I> {
    pub resolve: Option<Plugin>,

    #[serde(rename = "ls-remote")]
    pub ls_remote: Option<Plugin>,

    phantom: PhantomData<I>
}

impl Config {
    pub fn into_config(self) -> Fallible<config::Config> {
        Ok(config::Config {
            node: if let Some(n) = self.node {
                Some(n.into_tool_config()?)
            } else {
                None
            },
            yarn: if let Some(y) = self.yarn {
                Some(y.into_tool_config()?)
            } else {
                None
            },
        })
    }
}

impl<I: Install> ToolConfig<I> {
    pub fn into_tool_config(self) -> Fallible<config::ToolConfig<I>> {
        Ok(config::ToolConfig {
            resolve: if let Some(p) = self.resolve {
                Some(p.into_resolve()?)
            } else {
                None
            },
            ls_remote: if let Some(p) = self.ls_remote {
                Some(p.into_ls_remote()?)
            } else {
                None
            },
            phantom: PhantomData
        })
    }
}
