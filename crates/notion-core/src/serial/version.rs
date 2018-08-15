use notion_fail::{ExitCode, Fallible, NotionFail, ResultExt};
use semver::{ReqParseError, VersionReq};

#[derive(Fail, Debug)]
#[fail(display = "{}", error)]
pub(crate) struct VersionParseError {
    pub(crate) error: ReqParseError,
}

impl VersionParseError {
    pub(crate) fn from_req_parse_error(error: &ReqParseError) -> Self {
        VersionParseError {
            error: error.clone(),
        }
    }
}

impl_notion_fail!(VersionParseError, NoVersionMatch);

pub fn parse_requirements(src: &str) -> Fallible<VersionReq> {
    let src = src.trim();
    Ok(
        if src.len() > 0 && src.chars().next().unwrap().is_digit(10) {
            let defaulted = format!("={}", src);
            VersionReq::parse(&defaulted).with_context(VersionParseError::from_req_parse_error)?
        } else {
            VersionReq::parse(src).with_context(VersionParseError::from_req_parse_error)?
        },
    )
}
