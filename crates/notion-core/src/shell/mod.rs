use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use semver::Version;

use crate::fs::ensure_containing_dir_exists;
use notion_fail::{ExitCode, Fallible, NotionError, NotionFail, ResultExt};

use crate::env;

mod bash;

pub(crate) use self::bash::Bash;

pub enum Postscript {
    Activate(String),
    Deactivate(String),
    ToolVersion { tool: String, version: Version },
}

/// Thrown when the postscript file was not specified in the Notion environment.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Notion postscript file not specified")]
#[notion_fail(code = "EnvironmentError")]
struct UnspecifiedPostscriptError;

pub trait Shell {
    fn postscript_path(&self) -> &Path;

    fn compile_postscript(&self, postscript: &Postscript) -> String;

    fn save_postscript(&self, postscript: &Postscript) -> Fallible<()> {
        ensure_containing_dir_exists(&self.postscript_path())?;
        let mut file = File::create(self.postscript_path()).unknown()?;
        file.write_all(self.compile_postscript(postscript).as_bytes())
            .unknown()?;
        Ok(())
    }
}

pub struct CurrentShell(Box<dyn Shell>);

/// Thrown when the shell name was not specified in the Notion environment.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Notion shell not specified")]
#[notion_fail(code = "EnvironmentError")]
struct UnspecifiedShellError;

impl CurrentShell {
    pub fn detect() -> Fallible<Self> {
        match env::shell_name() {
            Some(name) => Ok(name.parse()?),
            None => {
                throw!(UnspecifiedShellError);
            }
        }
    }
}

impl Shell for CurrentShell {
    fn postscript_path(&self) -> &Path {
        let &CurrentShell(ref shell) = self;
        shell.postscript_path()
    }

    fn compile_postscript(&self, postscript: &Postscript) -> String {
        let &CurrentShell(ref shell) = self;
        shell.compile_postscript(postscript)
    }
}

/// Thrown when the shell name specified in the Notion environment is not supported.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Unrecognized command shell name: {}", name)]
#[notion_fail(code = "EnvironmentError")]
struct UnrecognizedShellError {
    name: String,
}

impl FromStr for CurrentShell {
    type Err = NotionError;

    fn from_str(src: &str) -> Result<Self, NotionError> {
        let postscript_path = env::postscript_path().ok_or(UnspecifiedPostscriptError)?;

        Ok(CurrentShell(match src {
            "bash" => Box::new(Bash { postscript_path }),
            _ => {
                throw!(UnrecognizedShellError {
                    name: src.to_string()
                });
            }
        }))
    }
}

#[cfg(test)]
pub mod tests {

    use super::{CurrentShell, Postscript, Shell};
    use semver::Version;
    use std::str::FromStr;

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
