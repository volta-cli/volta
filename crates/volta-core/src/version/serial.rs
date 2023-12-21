use node_semver::{Range, SemverError};

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
pub fn parse_requirements(src: &str) -> Result<Range, SemverError> {
    let src = src.trim().trim_start_matches('v');

    Range::parse(src)
}

#[cfg(test)]
pub mod tests {

    use crate::version::serial::parse_requirements;
    use node_semver::Range;

    #[test]
    fn test_parse_requirements() {
        assert_eq!(
            parse_requirements("1.2.3").unwrap(),
            Range::parse("=1.2.3").unwrap()
        );
        assert_eq!(
            parse_requirements("v1.5").unwrap(),
            Range::parse("=1.5").unwrap()
        );
        assert_eq!(
            parse_requirements("=1.2.3").unwrap(),
            Range::parse("=1.2.3").unwrap()
        );
        assert_eq!(
            parse_requirements("^1.2").unwrap(),
            Range::parse("^1.2").unwrap()
        );
        assert_eq!(
            parse_requirements(">=1.4").unwrap(),
            Range::parse(">=1.4").unwrap()
        );
        assert_eq!(
            parse_requirements("8.11 - 8.17 || 10.* || >= 12").unwrap(),
            Range::parse("8.11 - 8.17 || 10.* || >= 12").unwrap()
        );
    }
}
