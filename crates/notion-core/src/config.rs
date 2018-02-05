use std::str::FromStr;

use toml;

use path::user_config_file;
use failure;
use readext::ReadExt;
use untoml::touch;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub node: Option<NodeConfig>
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "node")]
pub struct NodeConfig {
    pub resolve: Option<PluginConfig>,

    #[serde(rename = "ls-remote")]
    pub ls_remote: Option<PluginConfig>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginConfig {
    Url(String),
    Bin(String)
}

pub fn config() -> Result<Config, failure::Error> {
    let path = user_config_file()?;
    let src = touch(&path)?.read_into_string()?;
    src.parse()
}

impl FromStr for Config {
    type Err = failure::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(toml::from_str(src)?)
    }
}
