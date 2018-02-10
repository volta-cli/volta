use semver::VersionReq;
use failure;

pub fn parse_req(src: &str) -> Result<VersionReq, failure::Error> {
    let src = src.trim();
    Ok(if src.len() > 0 && src.chars().next().unwrap().is_digit(10) {
        let defaulted = format!("={}", src);
        VersionReq::parse(&defaulted)?
    } else {
        VersionReq::parse(src)?
    })
}
