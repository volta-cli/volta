use std::fs::File;
use std::io::Write;

use semver::Version;

use notion_fail::{Fallible, ResultExt};

use env;

mod bash;

use self::bash::BashBackend;

pub enum Postscript {
    Path(String),
    ToolVersion { tool: String, version: Version }
}

impl Postscript {
    pub fn save(&self) -> Fallible<()> {
        Ok(match env::postscript_path() {
            Some(path) => {
                let mut file = File::create(path).unknown()?;
                // FIXME: determine the backend based on an env var communicated from the shell wrapper
                file.write_all(BashBackend.emit(self).as_bytes()).unknown()?;
            }
            None => unimplemented!()
        })
    }
}

trait Backend {
    fn emit(&self, postscript: &Postscript) -> String;
}
