use crate::error::ErrorDetails;
use crate::session::Session;
use semver::Version;
use volta_fail::Fallible;

/// Resolved Version of a Tool
pub enum ToolVersion {
    Node(Version),
    Npm(Version),
    Yarn(Version),
    Package(String, Version),
}

impl ToolVersion {
    pub fn fetch(self, _session: &mut Session) -> Fallible<()> {
        // TODO: Implement Fetch
        Err(ErrorDetails::Unimplemented {
            feature: "Fetching".into(),
        }
        .into())
    }
}
