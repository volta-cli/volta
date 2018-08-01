//! Provides error types for the installer tools.

use notion_fail::NotionFail;

use failure;

#[derive(Fail, Debug)]
#[fail(display = "{}", error)]
pub(crate) struct DownloadError {
    error: String,
}

impl DownloadError {
    pub(crate) fn from_error(error: &failure::Error) -> DownloadError {
        DownloadError {
            error: error.to_string(),
        }
    }
}

impl NotionFail for DownloadError {
    fn is_user_friendly(&self) -> bool {
        true
    }
    fn exit_code(&self) -> i32 {
        4
    }
}
