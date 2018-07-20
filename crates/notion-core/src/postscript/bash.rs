use super::{Postscript, Backend};

pub(crate) struct BashBackend;

impl Backend for BashBackend {
    fn emit(&self, postscript: &Postscript) -> String {
        match postscript {
            &Postscript::Path(ref s) => {
                // FIXME: proper escaping
                format!("export PATH='{}'\n", s)
            }
            &Postscript::ToolVersion { ref tool, ref version } => {
                format!("export NOTION_{}_VERSION={}\n", tool.to_ascii_uppercase(), version)
            }
        }
    }
}
