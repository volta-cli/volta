use semver::VersionReq;
use error::{Fallible, ResultExt};

pub fn parse_requirements(src: &str) -> Fallible<VersionReq> {
    let src = src.trim();
    Ok(if src.len() > 0 && src.chars().next().unwrap().is_digit(10) {
        let defaulted = format!("={}", src);
        VersionReq::parse(&defaulted).unknown()?
    } else {
        VersionReq::parse(src).unknown()?
    })
}
