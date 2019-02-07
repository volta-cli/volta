use semver::{ReqParseError, VersionReq};

pub fn parse_requirements(src: &str) -> Result<VersionReq, ReqParseError> {
    let src = src.trim();
    if src.len() > 0 && src.chars().next().unwrap().is_digit(10) {
        let defaulted = format!("={}", src);
        VersionReq::parse(&defaulted)
    } else if src.len() > 0 && src.chars().next().unwrap() == 'v' {
        let defaulted = src.replacen("v", "=", 1);
        VersionReq::parse(&defaulted)
    } else {
        VersionReq::parse(src)
    }
}

#[cfg(test)]
pub mod tests {

    use crate::version::serial::parse_requirements;
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
