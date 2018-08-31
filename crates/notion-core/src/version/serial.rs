use notion_fail::{ExitCode, Fallible, NotionFail, ResultExt};
use semver::{ReqParseError, VersionReq};

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "{}", error)]
#[notion_fail(code = "NoVersionMatch")]
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

pub fn parse_requirements(src: &str) -> Fallible<VersionReq> {
    let src = src.trim();
    Ok(
        if src.len() > 0 && src.chars().next().unwrap().is_digit(10) {
            let defaulted = format!("={}", src);
            VersionReq::parse(&defaulted).with_context(VersionParseError::from_req_parse_error)?
        } else if src.len() > 0 && src.chars().next().unwrap() == 'v' {
            let defaulted = src.replacen("v", "=", 1);
            VersionReq::parse(&defaulted).with_context(VersionParseError::from_req_parse_error)?
        } else {
            VersionReq::parse(src).with_context(VersionParseError::from_req_parse_error)?
        },
    )
}

#[cfg(test)]
pub mod tests {

    use version::serial::parse_requirements;
    use semver::VersionReq;

    #[test]
    fn test_parse_requirements() {
        assert_eq!(
            parse_requirements("1.2.3").unwrap(),
            VersionReq::parse("=1.2.3").unwrap()
        );
        assert_eq!(
            parse_requirements("v1.5").unwrap(),
            VersionReq::parse("=1.5").unwrap()
        );
        assert_eq!(
            parse_requirements("=1.2.3").unwrap(),
            VersionReq::parse("=1.2.3").unwrap()
        );
        assert_eq!(
            parse_requirements("^1.2").unwrap(),
            VersionReq::parse("^1.2").unwrap()
        );
        assert_eq!(
            parse_requirements(">=1.4").unwrap(),
            VersionReq::parse(">=1.4").unwrap()
        );
    }
}
