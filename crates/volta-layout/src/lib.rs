#[macro_use]
mod macros;

pub mod v0;
#[cfg(feature = "volta-updates")]
pub mod v1;

fn executable(name: &str) -> String {
    format!("{}{}", name, std::env::consts::EXE_SUFFIX)
}
