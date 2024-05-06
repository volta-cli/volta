use std::process::{Command, ExitCode};

pub fn main() -> ExitCode {
    let result = Command::new("volta-runner").args(std::env::args()).status();
    match result {
        Ok(exit_status) => match exit_status.code() {
            None => ExitCode::FAILURE,
            Some(code) => ExitCode::from(code as u8),
        },
        Err(_) => ExitCode::FAILURE,
    }
}
