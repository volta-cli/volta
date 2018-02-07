use std::str::FromStr;

use toml;

use path::user_config_file;
use failure;
use readext::ReadExt;
use untoml::touch;
use serial;
use plugin;

pub struct Config {
    pub node: Option<NodeConfig>
}

pub struct NodeConfig {
    pub resolve: Option<plugin::Resolve>,
    pub ls_remote: Option<plugin::LsRemote>
}

pub fn config() -> Result<Config, failure::Error> {
    let path = user_config_file()?;
    let src = touch(&path)?.read_into_string()?;
    src.parse()
}

impl FromStr for Config {
    type Err = failure::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let serial: serial::config::Config = toml::from_str(src)?;
        Ok(serial.into_config()?)
    }
}
