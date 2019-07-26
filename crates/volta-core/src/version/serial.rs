use semver::{Compat, ReqParseError, VersionReq};

// NOTE: using `parse_compat` here because the semver crate defaults to
// parsing in a cargo-compatible way. This is normally fine, except for
// 2 cases (that I know about):
//  * "1.2.3" parses as `^1.2.3` for cargo, but `=1.2.3` for Node
//  * `>1.2.3 <2.0.0` serializes to ">1.2.3, <2.0.0" for cargo (with the
//    comma), but ">1.2.3 <2.0.0" for Node (no comma, because Node parses
//    commas differently)
//
// Because we are parsing the version requirements from the command line,
// then serializing them to pass to `npm view`, they need to be handled in
// a Node-compatible way (or we get the wrong version info returned).
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
