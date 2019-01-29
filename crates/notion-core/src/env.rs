//! Provides utilities for extracting standard Notion environment variables.

use std::env;
use std::path::{Path, PathBuf};

pub const UNSAFE_GLOBAL: &'static str = "NOTION_UNSAFE_GLOBAL";

pub(crate) fn shell_name() -> Option<String> {
    env::var_os("NOTION_SHELL").map(|s| s.to_string_lossy().into_owned())
}

pub fn postscript_path() -> Option<PathBuf> {
    env::var_os("NOTION_POSTSCRIPT")
        .as_ref()
        .map(|ref s| Path::new(s).to_path_buf())
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn test_shell_name() {
        env::set_var("NOTION_SHELL", "bash");
        assert_eq!(shell_name().unwrap(), "bash".to_string());
    }

    #[test]
    fn test_postscript_path() {
        env::set_var("NOTION_POSTSCRIPT", "/some/path");
        assert_eq!(postscript_path().unwrap(), PathBuf::from("/some/path"));
    }

}
