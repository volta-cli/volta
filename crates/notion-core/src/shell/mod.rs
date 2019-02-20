use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use semver::Version;

use crate::error::ErrorDetails;
use crate::fs::ensure_containing_dir_exists;
use notion_fail::{throw, Fallible, NotionError, ResultExt};

use crate::env;

mod bash;
mod fish;

pub(crate) use self::bash::Bash;
pub(crate) use self::fish::Fish;

pub enum Postscript {
    Activate(String),
    Deactivate(String),
    ToolVersion { tool: String, version: Version },
}

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

impl CurrentShell {
    pub fn detect() -> Fallible<Self> {
        match env::shell_name() {
            Some(name) => Ok(name.parse()?),
            None => {
                throw!(ErrorDetails::UnspecifiedShell);
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

impl FromStr for CurrentShell {
    type Err = NotionError;

    fn from_str(src: &str) -> Result<Self, NotionError> {
        let postscript_path = env::postscript_path().ok_or(ErrorDetails::UnspecifiedPostscript)?;

        Ok(CurrentShell(match src {
            "bash" => Box::new(Bash { postscript_path }),
            "fish" => Box::new(Fish { postscript_path }),
            _ => {
                throw!(ErrorDetails::UnrecognizedShell {
                    name: src.to_string()
                });
            }
        }))
    }
}
