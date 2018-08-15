use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use semver::Version;

use notion_fail::{ExitCode, Fallible, NotionError, NotionFail, ResultExt};

use env;

mod bash;

pub(crate) use self::bash::Bash;

pub enum Postscript {
    Path(String),
    ToolVersion { tool: String, version: Version },
}

/// Thrown when the postscript file was not specified in the Notion environment.
#[derive(Fail, Debug)]
#[fail(display = "Notion postscript file not specified")]
struct UnspecifiedPostscriptError;

impl_notion_fail!(UnspecifiedPostscriptError, EnvironmentError);

pub trait Shell {
    fn postscript_path(&self) -> &Path;

    fn compile_postscript(&self, postscript: &Postscript) -> String;

    fn save_postscript(&self, postscript: &Postscript) -> Fallible<()> {
        let mut file = File::create(self.postscript_path()).unknown()?;
        file.write_all(self.compile_postscript(postscript).as_bytes())
            .unknown()?;
        Ok(())
    }
}

pub struct CurrentShell(Box<dyn Shell>);

/// Thrown when the shell name was not specified in the Notion environment.
#[derive(Fail, Debug)]
#[fail(display = "Notion shell not specified")]
struct UnspecifiedShellError;

impl_notion_fail!(UnspecifiedShellError, EnvironmentError);

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
#[derive(Fail, Debug)]
#[fail(display = "Unrecognized command shell name: {}", name)]
struct UnrecognizedShellError {
    name: String,
}

impl_notion_fail!(UnrecognizedShellError, EnvironmentError);

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
