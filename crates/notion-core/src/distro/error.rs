//! Provides error types for the installer tools.

use notion_fail::{ExitCode, NotionFail};

use failure;

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Failed to download version {}\n{}", version, error)]
#[notion_fail(code = "NetworkError")]
pub(crate) struct DownloadError {
    version: String,
    error: String,
}

impl DownloadError {
    pub(crate) fn for_version(version: String) -> impl FnOnce(&failure::Error) -> DownloadError {
        move |error| DownloadError {
            version: version,
            error: error.to_string(),
        }
    }
}
