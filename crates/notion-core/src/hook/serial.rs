use super::super::hook;

use notion_fail::{FailExt, Fallible};

#[derive(Serialize, Deserialize)]
pub struct ToolHook {
    prefix: Option<String>,
    template: Option<String>,
    bin: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PublishHook {
    url: Option<String>,
    bin: Option<String>,
}

#[derive(Fail, Debug)]
#[fail(display = "Hook contains more than one of 'prefix', 'template', or 'bin' fields")]
struct MultipleFieldsSpecified;

#[derive(Fail, Debug)]
#[fail(display = "Hook must contain either a 'prefix', 'template', or 'bin' field")]
struct NoFieldSpecified;

impl ToolHook {
    fn into_hook<H, P, T, B>(self, to_prefix: P, to_template: T, to_bin: B) -> Fallible<H>
    where
        P: FnOnce(String) -> H,
        T: FnOnce(String) -> H,
        B: FnOnce(String) -> H,
    {
        match self {
            ToolHook {
                prefix: Some(prefix),
                template: None,
                bin: None,
            } => Ok(to_prefix(prefix)),
            ToolHook {
                prefix: None,
                template: Some(template),
                bin: None,
            } => Ok(to_template(template)),
            ToolHook {
                prefix: None,
                template: None,
                bin: Some(bin),
            } => Ok(to_bin(bin)),
            ToolHook {
                prefix: None,
                template: None,
                bin: None,
            } => Err(NoFieldSpecified.unknown()),
            _ => Err(MultipleFieldsSpecified.unknown()),
        }
    }

    pub fn into_distro_hook(self) -> Fallible<hook::ToolDistroHook> {
        self.into_hook(
            hook::ToolDistroHook::Prefix,
            hook::ToolDistroHook::Template,
            hook::ToolDistroHook::Bin,
        )
    }

    pub fn into_metadata_hook(self) -> Fallible<hook::ToolMetadataHook> {
        self.into_hook(
            hook::ToolMetadataHook::Prefix,
            hook::ToolMetadataHook::Template,
            hook::ToolMetadataHook::Bin,
        )
    }
}

#[derive(Fail, Debug)]
#[fail(display = "Hook contains both 'url' and 'bin' fields")]
struct BothUrlAndBin;

#[derive(Fail, Debug)]
#[fail(display = "Hook must contain either a 'url' or 'bin' field")]
struct NeitherUrlNorBin;

impl PublishHook {
    pub fn into_publish(self) -> Fallible<hook::Publish> {
        match self {
            PublishHook {
                url: Some(url),
                bin: None,
            } => Ok(hook::Publish::Url(url)),
            PublishHook {
                url: None,
                bin: Some(bin),
            } => Ok(hook::Publish::Bin(bin)),
            PublishHook {
                url: None,
                bin: None,
            } => Err(NeitherUrlNorBin.unknown()),
            _ => Err(BothUrlAndBin.unknown()),
        }
    }
}
