use std::fs::write;
use std::path::Path;
use std::str::FromStr;

use semver::Version;

use crate::error::{CreatePostscriptErrorPath, ErrorDetails};
use crate::fs::ensure_containing_dir_exists;
use volta_fail::{Fallible, ResultExt, VoltaError};

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
        let path = self.postscript_path();
        ensure_containing_dir_exists(&path)?;
        write(path, self.compile_postscript(postscript).as_bytes()).with_context(|_| {
            let in_dir = path
                .parent()
                .map_or(CreatePostscriptErrorPath::Unknown, |p| {
                    CreatePostscriptErrorPath::Directory(p.to_path_buf())
                });
            ErrorDetails::CreatePostscriptError { in_dir }
        })
    }
}

pub struct CurrentShell(Box<dyn Shell>);

impl CurrentShell {
    pub fn detect() -> Fallible<Self> {
        env::shell_name()
            .ok_or(ErrorDetails::UnspecifiedShell.into())
            .and_then(|name| name.parse())
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
    type Err = VoltaError;

    fn from_str(src: &str) -> Result<Self, VoltaError> {
        let postscript_path = env::postscript_path().ok_or(ErrorDetails::UnspecifiedPostscript)?;

        match src {
            "bash" => Ok(CurrentShell(Box::new(Bash { postscript_path }))),
            "fish" => Ok(CurrentShell(Box::new(Fish { postscript_path }))),
            _ => Err(ErrorDetails::UnrecognizedShell {
                name: src.to_string(),
            }
            .into()),
        }
    }
}
