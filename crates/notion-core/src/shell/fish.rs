use std::path::{Path, PathBuf};

use super::{Postscript, Shell};

pub(crate) struct Fish {
    pub(crate) postscript_path: PathBuf,
}

impl Shell for Fish {
    fn postscript_path(&self) -> &Path {
        &self.postscript_path
    }

    fn compile_postscript(&self, postscript: &Postscript) -> String {
        match postscript {
            &Postscript::Activate(ref s) => {
                let updated_path = format!("set -x PATH \"{}\"\n", s);
                updated_path + "set -x NOTION_HOME \"$HOME/.notion\"\n"
            }
            // ISSUE(#99): proper escaping
            &Postscript::Deactivate(ref s) => {
                format!("set -x PATH \"{}\"\nset -e NOTION_HOME\n", s)
            }
            &Postscript::ToolVersion {
                ref tool,
                ref version,
            } => format!(
                "set -x NOTION_{}_VERSION {}\n",
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
        let fish = CurrentShell::from_str("fish").expect("Could not create fish shell");

        assert_eq!(
            fish.compile_postscript(&Postscript::Deactivate("some:path".to_string())),
            "set -x PATH \"some:path\"\nset -e NOTION_HOME\n"
        );

        // ISSUE(#99): proper escaping
        assert_eq!(
            fish.compile_postscript(&Postscript::Deactivate(
                "/path:/with:/single'quotes'".to_string()
            )),
            "set -x PATH \"/path:/with:/single'quotes'\"\nset -e NOTION_HOME\n"
        );

        assert_eq!(
            fish.compile_postscript(&Postscript::ToolVersion {
                tool: "test".to_string(),
                version: Version::parse("2.4.5").unwrap()
            }),
            "set -x NOTION_TEST_VERSION 2.4.5\n"
        );

        assert_eq!(
            fish.compile_postscript(&Postscript::Activate("some:path".to_string())),
            "set -x PATH \"some:path\"\nset -x NOTION_HOME \"$HOME/.notion\"\n"
        );
    }
}
