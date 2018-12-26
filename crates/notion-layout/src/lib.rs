extern crate notion_layout_macro;

pub mod v0;
pub mod v1;

pub(crate) fn executable(name: &str) -> String {
    format!("{}{}", name, std::env::consts::EXE_SUFFIX)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
