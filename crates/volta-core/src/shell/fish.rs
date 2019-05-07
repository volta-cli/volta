use std::path::{Path, PathBuf};

use super::{Postscript, Shell};

pub(crate) struct Fish {
    pub(crate) postscript_path: PathBuf,
}

static STATUS_HANDLING: &'static str = r#"
if test $status != 0;
  printf '\n\033[1;31mError\033[0m: Volta cannot update your `PATH`. If you are running fish 2.x, this often\n' 1>&2
  printf '       happens if your `PATH` includes an entry pointing to a value that is not\n' 1>&2
  printf '       a directory. For `volta deactivate` or `volta activate` to work, you\n' 1>&2
  printf '       must either change your `PATH` in \033[4m~/.config/fish/config.fish\033[0m so it only\n' 1>&2
  printf '       includes valid directories, or update Fish to at least 3.0.0.' 1>&2
  exit $status
end;
"#;

static SET_VOLTA_HOME: &'static str = "set -x VOLTA_HOME \"$HOME/.volta\"\n";
static UNSET_VOLTA_HOME: &'static str = "set -e VOLTA_HOME\n";

impl Shell for Fish {
    fn postscript_path(&self) -> &Path {
        &self.postscript_path
    }

    fn compile_postscript(&self, postscript: &Postscript) -> String {
        match postscript {
            &Postscript::Activate(ref s) => {
                let updated_path = format!("set -x PATH \"{}\"\n", s);
                updated_path + STATUS_HANDLING + SET_VOLTA_HOME
            }
            // ISSUE(#99): proper escaping
            &Postscript::Deactivate(ref s) => {
                let updated_path = format!("set -x PATH \"{}\"\n", s);
                updated_path + STATUS_HANDLING + UNSET_VOLTA_HOME
            }
            &Postscript::ToolVersion {
                ref tool,
                ref version,
            } => format!(
                "set -x VOLTA_{}_VERSION {}\n",
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
            String::from("set -x PATH \"some:path\"\n")
                + super::STATUS_HANDLING
                + super::UNSET_VOLTA_HOME
        );

        // ISSUE(#99): proper escaping
        assert_eq!(
            fish.compile_postscript(&Postscript::Deactivate(
                "/path:/with:/single'quotes'".to_string()
            )),
            String::from("set -x PATH \"/path:/with:/single'quotes'\"\n")
                + super::STATUS_HANDLING
                + super::UNSET_VOLTA_HOME
        );

        assert_eq!(
            fish.compile_postscript(&Postscript::ToolVersion {
                tool: "test".to_string(),
                version: Version::parse("2.4.5").unwrap()
            }),
            "set -x VOLTA_TEST_VERSION 2.4.5\n"
        );

        assert_eq!(
            fish.compile_postscript(&Postscript::Activate("some:path".to_string())),
            String::from("set -x PATH \"some:path\"\n")
                + super::STATUS_HANDLING
                + super::SET_VOLTA_HOME
        );
    }
}
