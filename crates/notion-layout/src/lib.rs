extern crate cfg_if;
extern crate notion_layout_macro;

pub mod v1;

pub(crate) fn executable(name: &str) -> String {
    format!("{}{}", name, std::env::consts::EXE_SUFFIX)
}
