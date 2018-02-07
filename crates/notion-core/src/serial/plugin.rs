use super::super::plugin;

use failure;
use semver::Version;

#[derive(Serialize, Deserialize)]
pub struct Plugin {
    url: Option<String>,
    bin: Option<String>
}

// FIXME: should abstract this

impl Plugin {
    pub fn into_resolve(self) -> Result<plugin::Resolve, failure::Error> {
        match self {
            Plugin { url: Some(_), bin: Some(_) } => {
                Err(format_err!("plugin contains both 'url' and 'bin' field"))
            }
            Plugin { url: Some(url), bin: None } => {
                Ok(plugin::Resolve::Url(url))
            }
            Plugin { url: None, bin: Some(bin) } => {
                Ok(plugin::Resolve::Bin(bin))
            }
            Plugin { url: None, bin: None } => {
                Err(format_err!("plugin must contain either a 'url' or 'bin' field"))
            }
        }
    }

    pub fn into_ls_remote(self) -> Result<plugin::LsRemote, failure::Error> {
        match self {
            Plugin { url: Some(_), bin: Some(_) } => {
                Err(format_err!("plugin contains both 'url' and 'bin' field"))
            }
            Plugin { url: Some(url), bin: None } => {
                Ok(plugin::LsRemote::Url(url))
            }
            Plugin { url: None, bin: Some(bin) } => {
                Ok(plugin::LsRemote::Bin(bin))
            }
            Plugin { url: None, bin: None } => {
                Err(format_err!("plugin must contain either a 'url' or 'bin' field"))
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ResolveResponse {
    url: String,
    version: String
}

impl ResolveResponse {
    pub fn into_resolve_response(self) -> Result<plugin::ResolveResponse, failure::Error> {
        Ok(plugin::ResolveResponse::Url {
            url: self.url,
            version: Version::parse(&self.version)?
        })
    }
}
