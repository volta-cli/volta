use super::super::plugin;

use failure::Fail;

use notion_fail::{FailExt, Fallible, ResultExt};
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Plugin {
    url: Option<String>,
    bin: Option<String>,
}

#[derive(Fail, Debug)]
#[fail(display = "Plugin contains both 'url' and 'bin' fields")]
struct BothUrlAndBin;

#[derive(Fail, Debug)]
#[fail(display = "Plugin must contain either a 'url' or 'bin' field")]
struct NeitherUrlNorBin;

impl Plugin {
    fn into_plugin<T, U, B>(self, to_url: U, to_bin: B) -> Fallible<T>
    where
        U: FnOnce(String) -> T,
        B: FnOnce(String) -> T,
    {
        match self {
            Plugin {
                url: Some(_),
                bin: Some(_),
            } => Err(BothUrlAndBin.unknown()),
            Plugin {
                url: Some(url),
                bin: None,
            } => Ok(to_url(url)),
            Plugin {
                url: None,
                bin: Some(bin),
            } => Ok(to_bin(bin)),
            Plugin {
                url: None,
                bin: None,
            } => Err(NeitherUrlNorBin.unknown()),
        }
    }

    pub fn into_resolve(self) -> Fallible<plugin::ResolvePlugin> {
        self.into_plugin(plugin::ResolvePlugin::Url, plugin::ResolvePlugin::Bin)
    }

    pub fn into_ls_remote(self) -> Fallible<plugin::LsRemote> {
        self.into_plugin(plugin::LsRemote::Url, plugin::LsRemote::Bin)
    }

    pub fn into_publish(self) -> Fallible<plugin::Publish> {
        self.into_plugin(plugin::Publish::Url, plugin::Publish::Bin)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ResolveResponse {
    version: String,
    url: Option<String>,
    stream: Option<bool>,
}

#[derive(Fail, Debug)]
#[fail(display = "Plugin contains both 'url' and 'stream' fields")]
struct BothUrlAndStream;

#[derive(Fail, Debug)]
#[fail(display = "Plugin must contain either a 'url' or 'stream' field")]
struct NeitherUrlNorStream;

#[derive(Fail, Debug)]
#[fail(display = "Plugin 'stream' field must be 'true' if present")]
struct FalseStream;

impl ResolveResponse {
    pub fn into_resolve_response(self) -> Fallible<plugin::ResolveResponse> {
        match self {
            ResolveResponse {
                url: Some(_),
                stream: Some(_),
                ..
            } => Err(BothUrlAndStream.unknown()),
            ResolveResponse {
                url: None,
                stream: None,
                ..
            } => Err(NeitherUrlNorStream.unknown()),
            ResolveResponse {
                url: None,
                stream: Some(false),
                ..
            } => Err(FalseStream.unknown()),
            ResolveResponse {
                url: Some(url),
                stream: None,
                version,
            } => Ok(plugin::ResolveResponse::Url {
                url,
                version: Version::parse(&version).unknown()?,
            }),
            ResolveResponse {
                url: None,
                stream: Some(true),
                version,
            } => Ok(plugin::ResolveResponse::Stream {
                version: Version::parse(&version).unknown()?,
            }),
        }
    }
}
