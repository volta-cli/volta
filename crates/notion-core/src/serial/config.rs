use super::super::config;

use failure;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub node: Option<NodeConfig>
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "node")]
pub struct NodeConfig {
    pub resolve: Option<Plugin>,

    #[serde(rename = "ls-remote")]
    pub ls_remote: Option<Plugin>
}

#[derive(Serialize, Deserialize)]
pub struct Plugin {
    url: Option<String>,
    bin: Option<String>
}

impl Config {
    pub fn into_config(self) -> Result<config::Config, failure::Error> {
        Ok(config::Config {
            node: if let Some(n) = self.node {
                Some(n.into_node_config()?)
            } else {
                None
            }
        })
    }
}

impl NodeConfig {
    pub fn into_node_config(self) -> Result<config::NodeConfig, failure::Error> {
        Ok(config::NodeConfig {
            resolve: if let Some(p) = self.resolve {
                Some(p.into_plugin()?)
            } else {
                None
            },
            ls_remote: if let Some(p) = self.ls_remote {
                Some(p.into_plugin()?)
            } else {
                None
            }
        })
    }
}

impl Plugin {
    pub fn into_plugin(self) -> Result<config::Plugin, failure::Error> {
        match self {
            Plugin { url: Some(_), bin: Some(_) } => {
                Err(format_err!("plugin contains both 'url' and 'bin' field"))
            }
            Plugin { url: Some(url), bin: None } => {
                Ok(config::Plugin::Url(url))
            }
            Plugin { url: None, bin: Some(bin) } => {
                Ok(config::Plugin::Bin(bin))
            }
            Plugin { url: None, bin: None } => {
                Err(format_err!("plugin must contain either a 'url' or 'bin' field"))
            }
        }
    }
}
