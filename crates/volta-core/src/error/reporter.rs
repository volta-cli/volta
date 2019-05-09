use std::env::{self, args_os};
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::PathBuf;

use crate::fs::ensure_containing_dir_exists;
use crate::path::log_dir;
use crate::style::{format_error_cause, format_error_message};
use chrono::Local;
use failure::Error;
use volta_fail::VoltaError;

const VOLTA_DEV: &'static str = "VOLTA_DEV";

/// Represents the context from which an error is being reported.
pub enum ErrorContext {
    /// An error reported from the `volta` executable.
    Volta,

    /// An error reported from a shim.
    Shim,
}

/// Reporter for showing errors to the terminal and error logs
pub struct ErrorReporter {
    /// Volta version to display in error logs
    version: String,

    /// Flag indicating whether to report additional details to the terminal
    verbose: bool,
}

impl ErrorReporter {
    /// Create a new ErrorReporter from a verbose flag
    pub fn from_flag(volta_version: &str, verbose: bool) -> Self {
        if verbose {
            ErrorReporter {
                version: volta_version.to_string(),
                verbose,
            }
        } else {
            ErrorReporter::from_env(volta_version)
        }
    }

    /// Create a new ErrorReporter from the environment variables
    pub fn from_env(volta_version: &str) -> Self {
        ErrorReporter {
            version: volta_version.to_string(),
            verbose: env::var(VOLTA_DEV).is_ok(),
        }
    }

    /// Report an error, both to the terminal and the error log
    pub fn report(&self, cx: ErrorContext, err: &VoltaError) {
        let message = format_error_message(cx, err);

        eprintln!("{}", message);

        if let Some(details) = compose_error_details(err) {
            if self.verbose {
                eprintln!();
                eprintln!("{}", details);
            }

            match self.write_error_log(message, details) {
                Ok(log_file) => {
                    eprintln!("Error details written to: {}", log_file.to_string_lossy());
                }
                Err(_) => {
                    eprintln!("Unable to write error log!");
                }
            }
        }
    }

    /// Write an error log with additional details about the error
    fn write_error_log(&self, message: String, details: String) -> Result<PathBuf, Error> {
        let file_name = Local::now()
            .format("volta-error-%Y-%m-%d_%H_%M_%S%.3f.log")
            .to_string();
        let log_file_path = log_dir()?.join(&file_name);

        ensure_containing_dir_exists(&log_file_path)?;
        let mut log_file = File::create(&log_file_path)?;

        writeln!(log_file, "{}", collect_arguments())?;
        writeln!(log_file, "Volta v{}", self.version)?;
        writeln!(log_file)?;
        writeln!(log_file, "{}", message)?;
        writeln!(log_file)?;
        writeln!(log_file, "{}", details)?;

        Ok(log_file_path)
    }
}

fn compose_error_details(err: &VoltaError) -> Option<String> {
    // Only compose details if there is an underlying cause for the error
    let mut current = match err.as_fail().cause() {
        Some(cause) => cause,
        None => {
            return None;
        }
    };
    let mut details = String::new();

    // Walk up the tree of causes and include all of them
    loop {
        details.push_str(&format_error_cause(current));

        match current.cause() {
            Some(cause) => {
                details.push_str("\n\n");
                current = cause;
            }
            None => {
                break;
            }
        };
    }

    // ISSUE #75 - Once we have a way to determine backtraces without RUST_BACKTRACE, we can make this always available
    // Until then, we know that if the env var is not set, the backtrace will be empty
    if env::var("RUST_BACKTRACE").is_ok() {
        // Note: The implementation of `Display` for Backtrace includes a 'stack backtrace:' prefix
        write!(details, "\n\n{}", err.backtrace()).expect("write! to a String doesn't fail");
    }

    Some(details)
}

/// Combines all the arguments into a single String
fn collect_arguments() -> String {
    // The Debug formatter for OsString properly quotes and escapes each value
    args_os()
        .map(|arg| format!("{:?}", arg))
        .collect::<Vec<String>>()
        .join(" ")
}
