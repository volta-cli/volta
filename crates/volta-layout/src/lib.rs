#[macro_use]
mod macros;

pub mod v0;
pub mod v1;
pub mod v2;
pub mod v3;
#[cfg(feature = "pnpm")]
pub mod v4;

fn executable(name: &str) -> String {
    format!("{}{}", name, std::env::consts::EXE_SUFFIX)
}
