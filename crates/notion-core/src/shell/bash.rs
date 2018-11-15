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
