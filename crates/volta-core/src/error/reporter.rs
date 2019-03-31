use std::env::{self, args_os};
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::PathBuf;

use crate::fs::ensure_containing_dir_exists;
use crate::layout::layout;
use crate::style::{format_error_cause, format_error_message};
use chrono::Local;
use failure::Error;
use log::{debug, error};
use volta_fail::VoltaError;

/// Report an error, both to the console and to error logs
pub fn report_error(volta_version: &str, err: &VoltaError) {
    let message = err.to_string();
    error!("{}", message);

    if let Some(details) = compose_error_details(err) {
        debug!("{}", details);

        // Note: Writing the error log info directly to stderr as it is a message for the user
        // Any custom logs will have all of the details already, so showing a message about writing
        // the error log would be redundant
        match write_error_log(volta_version, message, details) {
            Ok(log_file) => {
                eprintln!("Error details written to {}", log_file.to_string_lossy());
            }
            Err(_) => {
                eprintln!("Unable to write error log!");
            }
        }
    }
}

/// Write an error log with all details about the error
fn write_error_log(
    volta_version: &str,
    message: String,
    details: String,
) -> Result<PathBuf, Error> {
    let file_name = Local::now()
        .format("volta-error-%Y-%m-%d_%H_%M_%S%.3f.log")
        .to_string();
    let log_file_path = layout()?.user.log_dir().join(&file_name);

    ensure_containing_dir_exists(&log_file_path)?;
    let mut log_file = File::create(&log_file_path)?;

    writeln!(log_file, "{}", collect_arguments())?;
    writeln!(log_file, "Volta v{}", volta_version)?;
    writeln!(log_file)?;
    writeln!(log_file, "{}", message)?;
    writeln!(log_file)?;
    writeln!(log_file, "{}", details)?;

    Ok(log_file_path)
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
