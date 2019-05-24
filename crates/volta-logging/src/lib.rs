//! This crate provides custom Logger implementations for use with the `log` crate
use atty::Stream;
use console::style;
use log::{Level, Log, Metadata, Record, SetLoggerError};
use std::env;
use std::fmt::Display;
use textwrap::{NoHyphenation, Wrapper};

const ERROR_PREFIX: &'static str = "error:";
const WARNING_PREFIX: &'static str = "warning:";
const SHIM_ERROR_PREFIX: &'static str = "Volta error:";
const SHIM_WARNING_PREFIX: &'static str = "Volta warning:";
const VOLTA_DEV: &'static str = "VOLTA_DEV";
const ALLOWED_CRATE: &'static str = "volta_core";
const MAX_WIDTH: usize = 100;
const WRAP_INDENT: &'static str = "    ";

/// Gets the terminal width, capped at MAX_WIDTH, or None if we aren't running in a terminal
pub fn text_width() -> Option<usize> {
    term_size::dimensions().map(|(w, _)| w.min(MAX_WIDTH))
}

/// Represents the context from which the logger was created
pub enum LoggerContext {
    /// Log messages from the `volta` executable
    Volta,

    /// Log messages from one of the shims
    Shim,
}

pub struct VoltaLogger {
    context: LoggerContext,
    level: Level,
}

impl Log for VoltaLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) && record.target().starts_with(ALLOWED_CRATE) {
            match record.level() {
                Level::Error => self.log_error(record.args()),
                Level::Warn => self.log_warning(record.args()),
                _ => println!("{}", record.args()),
            }
        }
    }

    fn flush(&self) {}
}

impl VoltaLogger {
    fn log_error<D>(&self, message: &D)
    where
        D: Display,
    {
        let prefix = match &self.context {
            LoggerContext::Volta => ERROR_PREFIX,
            LoggerContext::Shim => SHIM_ERROR_PREFIX,
        };

        eprintln!("{} {}", style(prefix).red().bold(), message);
    }

    fn log_warning<D>(&self, message: &D)
    where
        D: Display,
    {
        let prefix = match &self.context {
            LoggerContext::Volta => WARNING_PREFIX,
            LoggerContext::Shim => SHIM_WARNING_PREFIX,
        };

        println!(
            "{}{}",
            style(prefix).yellow().bold(),
            wrap_content(prefix, message)
        );
    }

    /// Initialize the global logger with a VoltaLogger instance
    /// If the Verbose flag is set, level is set to Debug
    /// Otherwise will use environment information to set the log level
    pub fn init_from_flag(context: LoggerContext, verbose: bool) -> Result<(), SetLoggerError> {
        let level = if verbose {
            Level::Debug
        } else {
            level_from_env()
        };

        VoltaLogger::init(context, level)
    }

    /// Initialize the global logger using the environment information
    pub fn init_from_env(context: LoggerContext) -> Result<(), SetLoggerError> {
        VoltaLogger::init(context, level_from_env())
    }

    fn init(context: LoggerContext, level: Level) -> Result<(), SetLoggerError> {
        let logger = VoltaLogger { context, level };
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(level.to_level_filter());
        Ok(())
    }
}

/// Wraps the supplied content to the terminal width, if we are in a terminal.
/// If not, returns the content as a String
///
/// Note: Uses the supplied prefix to calculate the terminal width, but then removes
/// it so that it can be styled (style characters are counted against the wrapped width)
fn wrap_content<D>(prefix: &str, content: &D) -> String
where
    D: Display,
{
    match text_width() {
        Some(width) => Wrapper::with_splitter(width, NoHyphenation)
            .subsequent_indent(WRAP_INDENT)
            .break_words(false)
            .fill(&format!("{} {}", prefix, content))
            .replace(prefix, ""),
        None => format!(" {}", content),
    }
}

/// Determines the correct logging level based on the environment
/// If VOLTA_DEV is set then we use Debug, which is the same as setting --verbose
/// If not, we check the current stdout for whether it is a TTY
///     If it is a TTY, we use Info
///     If it is NOT a TTY, we use Error as we don't want to show warnings when running as a script
fn level_from_env() -> Level {
    match (env::var(VOLTA_DEV), atty::is(Stream::Stdout)) {
        (Ok(_), _) => Level::Debug,
        (_, true) => Level::Info,
        (_, false) => Level::Error,
    }
}

#[cfg(test)]
mod tests {}
