use std::str::FromStr;

use toml::value::{Value, Table};

use path::user_config_file;
use untoml::{ParseToml, Extract, load};
use failure;

use super::ConfigError;

fn toml_error(key: String) -> ConfigError {
    ConfigError { key }
}

pub struct Config {
    pub node: Option<NodeConfig>
}

pub struct NodeConfig {
    pub resolve: Option<String>,
    pub fetch: Option<NodeFetchConfig>
}

pub enum NodeFetchConfig {
    Url(String),
    Bin(String)
}

pub fn config() -> Result<Config, failure::Error> {
    let path = user_config_file()?;
    load(&path)
}

fn parse_node_fetch_config(node: &mut Table) -> Result<Option<NodeFetchConfig>, failure::Error> {
    if node.contains_key("url") {
        Ok(Some(NodeFetchConfig::Url(node.extract("url", toml_error)?.string("node.resolve.url", toml_error)?)))
    } else if node.contains_key("bin") {
        Ok(Some(NodeFetchConfig::Bin(node.extract("bin", toml_error)?.string("node.resolve.bin", toml_error)?)))
    } else {
        Ok(None)
    }
}

fn parse_node_config(root: &mut Table) -> Result<Option<NodeConfig>, failure::Error> {
    if !root.contains_key("node") {
        return Ok(None);
    }
    let mut node = root.extract("node", toml_error)?.table("node", toml_error)?;

    let resolve = if !node.contains_key("resolve") {
        None
    } else {
        Some(node.extract("resolve", toml_error)?.string("node.resolve", toml_error)?)
    };

    let fetch = if !node.contains_key("fetch") {
        None
    } else {
        parse_node_fetch_config(&mut node.extract("fetch", toml_error)?.table("node.fetch", toml_error)?)?
    };

    Ok(Some(NodeConfig { resolve, fetch }))
}

impl FromStr for Config {
    type Err = failure::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let toml = src.parse::<Value>()?;
        let mut root = toml.table("<root>", toml_error)?;
        let node = parse_node_config(&mut root)?;
        Ok(Config { node })
    }
}
