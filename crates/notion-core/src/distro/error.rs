//! Provides error types for the installer tools.

use notion_fail::{ExitCode, NotionFail};

use failure;

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Failed to download version {} from {}\n{}", version, from_url, error)]
#[notion_fail(code = "NetworkError")]
pub(crate) struct DownloadError {
    version: String,
    from_url: String,
    error: String,
}

impl DownloadError {
    pub(crate) fn for_version(
        version: String,
        from_url: String,
    ) -> impl FnOnce(&failure::Error) -> DownloadError {
        move |error| DownloadError {
            version: version,
            from_url: from_url,
            error: error.to_string(),
        }
    }
}
