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
            &Postscript::Path(ref s) => {
                format!("set -U fish_user_paths '{}'\n $fish_user_paths", s)
            }
            &Postscript::ToolVersion {
                ref tool,
                ref version,
            } => format!(
                "set -U NOTION_{}_VERSION {}\n",
                tool.to_ascii_uppercase(),
                version
            ),
        }
    }
}
