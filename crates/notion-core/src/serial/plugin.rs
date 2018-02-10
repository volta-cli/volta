use super::super::plugin;

use failure;
use semver::Version;

#[derive(Serialize, Deserialize)]
pub struct Plugin {
    url: Option<String>,
    bin: Option<String>
}

impl Plugin {
    fn into_plugin<T, U, B>(self, to_url: U, to_bin: B) -> Result<T, failure::Error>
      where U: FnOnce(String) -> T,
            B: FnOnce(String) -> T
    {
        match self {
            Plugin { url: Some(_), bin: Some(_) } => {
                Err(format_err!("plugin contains both 'url' and 'bin' field"))
            }
            Plugin { url: Some(url), bin: None } => {
                Ok(to_url(url))
            }
            Plugin { url: None, bin: Some(bin) } => {
                Ok(to_bin(bin))
            }
            Plugin { url: None, bin: None } => {
                Err(format_err!("plugin must contain either a 'url' or 'bin' field"))
            }
        }
    }

    pub fn into_resolve(self) -> Result<plugin::Resolve, failure::Error> {
        self.into_plugin(plugin::Resolve::Url, plugin::Resolve::Bin)
    }

    pub fn into_ls_remote(self) -> Result<plugin::LsRemote, failure::Error> {
        self.into_plugin(plugin::LsRemote::Url, plugin::LsRemote::Bin)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ResolveResponse {
    version: String,
    url: Option<String>,
    stream: Option<bool>
}

impl ResolveResponse {
    pub fn into_resolve_response(self) -> Result<plugin::ResolveResponse, failure::Error> {
        match self {
            ResolveResponse { url: Some(_), stream: Some(_), .. } => {
                Err(format_err!("response cannot contain both 'url' and 'stream' fields"))
            }
            ResolveResponse { url: None, stream: None, .. } => {
                Err(format_err!("response must contain either 'url' or 'stream' field"))
            }
            ResolveResponse { url: None, stream: Some(false), .. } => {
                Err(format_err!("'stream' field must be 'true' if present"))
            }
            ResolveResponse { url: Some(url), stream: None, version } => {
                Ok(plugin::ResolveResponse::Url { url, version: Version::parse(&version)? })
            }
            ResolveResponse { url: None, stream: Some(true), version } => {
                Ok(plugin::ResolveResponse::Stream { version: Version::parse(&version)? })
            }
        }
    }
}
