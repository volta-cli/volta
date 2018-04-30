use super::super::config;

use super::plugin::Plugin;

use notion_fail::Fallible;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub node: Option<NodeConfig>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "node")]
pub struct NodeConfig {
    pub resolve: Option<Plugin>,

    #[serde(rename = "ls-remote")]
    pub ls_remote: Option<Plugin>,
}

impl Config {
    pub fn into_config(self) -> Fallible<config::Config> {
        Ok(config::Config {
            node: if let Some(n) = self.node {
                Some(n.into_node_config()?)
            } else {
                None
            },
        })
    }
}

impl NodeConfig {
    pub fn into_node_config(self) -> Fallible<config::NodeConfig> {
        Ok(config::NodeConfig {
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
        })
    }
}
