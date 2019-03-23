use std::env::{self, args_os};
use std::fs::File;
use std::io::Write;

use crate::fs::ensure_containing_dir_exists;
use crate::path::log_dir;
use crate::style::{format_error_details, format_error_message};
use chrono::Local;
use failure::Error;
use notion_fail::NotionError;

const NOTION_DEV: &'static str = "NOTION_DEV";

/// Represents the context from which an error is being reported.
pub enum ErrorContext {
    /// An error reported from the `notion` executable.
    Notion,

    /// An error reported from a shim.
    Shim,
}

pub enum ErrorReporter {
    /// Reports errors in the standard, concise format
    Standard,

    /// Reports errors with additional details
    Verbose,
}

impl ErrorReporter {
    /// Create a new ErrorReporter of the default type
    pub fn new() -> Self {
        if env::var(NOTION_DEV).is_ok() {
            ErrorReporter::Verbose
        } else {
            ErrorReporter::Standard
        }
    }

    /// Create a new verbose ErrorReporter
    pub fn verbose() -> Self {
        ErrorReporter::Verbose
    }

    /// Report an error, both to the terminal and the error log
    pub fn report(&self, cx: ErrorContext, err: &NotionError) {
        let message = format_error_message(cx, err);
        let details = format_error_details(err);

        match self {
            ErrorReporter::Standard => eprint!("{}", message),
            ErrorReporter::Verbose => eprint!("{}{}", message, details),
        }

        match write_error_log(message, details) {
            Ok(log_file) => {
                eprintln!("Error log written to: {}", log_file);
            }
            Err(_) => {
                eprintln!("Unable to write error log!");
            }
        }
    }
}

fn write_error_log(message: String, details: String) -> Result<String, Error> {
    let file_name = Local::now()
        .format("notion-error-%Y-%m-%d_%H_%M_%S%.3f.log")
        .to_string();
    let log_file_path = log_dir()?.join(&file_name);

    ensure_containing_dir_exists(&log_file_path)?;
    let mut log_file = File::create(log_file_path)?;

    writeln!(log_file, "{}", collect_arguments())?;
    writeln!(log_file, "Notion v{}", env!("CARGO_PKG_VERSION"))?;
    writeln!(log_file)?;
    write!(log_file, "{}{}", message, details)?;

    Ok(file_name)
}

/// Combines all the arguments into a single String
fn collect_arguments() -> String {
    args_os()
        .map(|arg| arg.into_string().unwrap_or(String::from("<UNKNOWN>")))
        .collect::<Vec<String>>()
        .join(" ")
}
