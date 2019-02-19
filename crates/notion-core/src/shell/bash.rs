use std::path::{Path, PathBuf};

use super::{Postscript, Shell};

pub(crate) struct Bash {
    pub(crate) postscript_path: PathBuf,
}

impl Shell for Bash {
    fn postscript_path(&self) -> &Path {
        &self.postscript_path
    }

    fn compile_postscript(&self, postscript: &Postscript) -> String {
        match postscript {
            &Postscript::Activate(ref s) => format!(
                "export PATH='{}'\nexport NOTION_HOME=\"${{HOME}}/.notion\"\n",
                s
            ),
            &Postscript::Deactivate(ref s) => {
                // ISSUE(#99): proper escaping
                format!("export PATH='{}'\nunset NOTION_HOME\n", s)
            }
            &Postscript::ToolVersion {
                ref tool,
                ref version,
            } => format!(
                "export NOTION_{}_VERSION={}\n",
                tool.to_ascii_uppercase(),
                version
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;
    use std::str::FromStr;

    use crate::shell::{CurrentShell, Postscript, Shell};

    #[test]
    fn test_compile_postscript() {
        let bash = CurrentShell::from_str("bash").expect("Could not create bash shell");

        assert_eq!(
            bash.compile_postscript(&Postscript::Deactivate("some:path".to_string())),
            "export PATH='some:path'\nunset NOTION_HOME\n"
        );

        // ISSUE(#99): proper escaping
        assert_eq!(
            bash.compile_postscript(&Postscript::Deactivate(
                "/path:/with:/single'quotes'".to_string()
            )),
            "export PATH='/path:/with:/single'quotes''\nunset NOTION_HOME\n"
        );

        assert_eq!(
            bash.compile_postscript(&Postscript::ToolVersion {
                tool: "test".to_string(),
                version: Version::parse("2.4.5").unwrap()
            }),
            "export NOTION_TEST_VERSION=2.4.5\n"
        );

        assert_eq!(
            bash.compile_postscript(&Postscript::Activate("some:path".to_string())),
            "export PATH='some:path'\nexport NOTION_HOME=\"${HOME}/.notion\"\n"
        );
    }
}
