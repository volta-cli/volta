use semver::{Compat, ReqParseError, VersionReq};

pub fn parse_requirements(src: &str) -> Result<VersionReq, ReqParseError> {
    let src = src.trim();
    if src.len() > 0 && src.chars().next().unwrap().is_digit(10) {
        let defaulted = format!("={}", src);
        VersionReq::parse_compat(&defaulted, Compat::Node)
    } else if src.len() > 0 && src.chars().next().unwrap() == 'v' {
        let defaulted = src.replacen("v", "=", 1);
        VersionReq::parse_compat(&defaulted, Compat::Node)
    } else {
        VersionReq::parse_compat(src, Compat::Node)
    }
}

#[cfg(test)]
pub mod tests {

    use crate::version::serial::parse_requirements;
    use semver::{Compat, VersionReq};

    #[test]
    fn test_parse_requirements() {
        assert_eq!(
            parse_requirements("1.2.3").unwrap(),
            VersionReq::parse_compat("=1.2.3", Compat::Node).unwrap()
        );
        assert_eq!(
            parse_requirements("v1.5").unwrap(),
            VersionReq::parse_compat("=1.5", Compat::Node).unwrap()
        );
        assert_eq!(
            parse_requirements("=1.2.3").unwrap(),
            VersionReq::parse_compat("=1.2.3", Compat::Node).unwrap()
        );
        assert_eq!(
            parse_requirements("^1.2").unwrap(),
            VersionReq::parse_compat("^1.2", Compat::Node).unwrap()
        );
        assert_eq!(
            parse_requirements(">=1.4").unwrap(),
            VersionReq::parse_compat(">=1.4", Compat::Node).unwrap()
        );
    }
}
