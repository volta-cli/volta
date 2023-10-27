use std::process::{Command, ExitCode};

pub fn main() -> ExitCode {
    let result = Command::new("volta-runner").args(std::env::args()).status();
    match result {
        Ok(_) => ExitCode::code,
        Err(_) => ExitCode::FAILURE,
    }
}
