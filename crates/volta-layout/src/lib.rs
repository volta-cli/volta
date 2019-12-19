#[macro_use]
mod macros;

pub mod v0;
pub mod v1;

fn executable(name: &str) -> String {
    format!("{}{}", name, std::env::consts::EXE_SUFFIX)
}
