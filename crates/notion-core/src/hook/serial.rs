use super::super::hook;

use notion_fail::{FailExt, Fallible, ResultExt};
use semver::Version;

#[derive(Serialize, Deserialize)]
pub struct Hook {
    url: Option<String>,
    bin: Option<String>,
}

#[derive(Fail, Debug)]
#[fail(display = "Plugin contains both 'url' and 'bin' fields")]
struct BothUrlAndBin;

#[derive(Fail, Debug)]
#[fail(display = "Plugin must contain either a 'url' or 'bin' field")]
struct NeitherUrlNorBin;

impl Hook {
    fn into_hook<T, U, B>(self, to_url: U, to_bin: B) -> Fallible<T>
    where
        U: FnOnce(String) -> T,
        B: FnOnce(String) -> T,
    {
        match self {
            Hook {
                url: Some(_),
                bin: Some(_),
            } => Err(BothUrlAndBin.unknown()),
            Hook {
                url: Some(url),
                bin: None,
            } => Ok(to_url(url)),
            Hook {
                url: None,
                bin: Some(bin),
            } => Ok(to_bin(bin)),
            Hook {
                url: None,
                bin: None,
            } => Err(NeitherUrlNorBin.unknown()),
        }
    }

    pub fn into_resolve(self) -> Fallible<hook::ResolveHook> {
        self.into_hook(hook::ResolveHook::Url, hook::ResolveHook::Bin)
    }

    pub fn into_ls_remote(self) -> Fallible<hook::LsRemote> {
        self.into_hook(hook::LsRemote::Url, hook::LsRemote::Bin)
    }

    pub fn into_publish(self) -> Fallible<hook::Publish> {
        self.into_hook(hook::Publish::Url, hook::Publish::Bin)
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
    pub fn into_resolve_response(self) -> Fallible<hook::ResolveResponse> {
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
            } => Ok(hook::ResolveResponse::Url {
                url,
                version: Version::parse(&version).unknown()?,
            }),
            ResolveResponse {
                url: None,
                stream: Some(true),
                version,
            } => Ok(hook::ResolveResponse::Stream {
                version: Version::parse(&version).unknown()?,
            }),
        }
    }
}
