#[macro_use]
pub mod macros;

pub mod v0;

fn executable(name: &str) -> String {
    format!("{}{}", name, std::env::consts::EXE_SUFFIX)
}
