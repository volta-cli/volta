//! The view layer of Notion, with utilities for styling command-line output.

use std::env;
use std::fmt::Write;

use crate::error::ErrorContext;
use archive::Origin;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use notion_fail::NotionError;
use term_size;

// ISSUE #306 - When unknown error messages are removed, this can be removed as well
const INTERNAL_ERROR_MESSAGE: &'static str = "an internal error occurred

Notion is still a pre-alpha project, so we expect to run into some bugs,
but we'd love to hear about them so we can fix them!

Please feel free to reach out to us at \x1b[36m\x1b[1m@notionjs\x1b[0m on Twitter or file an issue at:

    \x1b[1mhttps://github.com/notion-cli/notion/issues\x1b[0m
";

macro_rules! write_str {
    ( $( $x:expr ),* ) => {
        write!($( $x, )*).expect("write! with String cannot fail")
    }
}

macro_rules! writeln_str {
    ( $( $x:expr ),* ) => {
        writeln!($( $x, )*).expect("write! with String cannot fail")
    }
}

/// Formats the error message to a string
pub(crate) fn format_error_message(cx: ErrorContext, err: &NotionError) -> String {
    let mut message = String::with_capacity(100);

    format_error_prefix(&mut message, cx);
    if err.is_user_friendly() {
        writeln_str!(message, "{}", err);
    } else {
        writeln_str!(message, "{}", INTERNAL_ERROR_MESSAGE);
    }

    message
}

/// Formats verbose error details to string
pub(crate) fn format_error_details(err: &NotionError) -> String {
    let mut details = String::new();

    format_error_cause(&mut details, err);
    format_error_backtrace(&mut details, err);

    details
}

/// Formats a styled prefix for an error
fn format_error_prefix(msg: &mut String, cx: ErrorContext) {
    match cx {
        ErrorContext::Notion => {
            // Since the command here was `notion`, it would be redundant to say that this was
            // a Notion error, so we are less explicit in the heading.
            write_str!(msg, "{} ", style("error:").red().bold());
        }
        ErrorContext::Shim => {
            // Since a Notion error is rare case for a shim, it can be surprising to a user.
            // To make it extra clear that this was a failure that happened in Notion when
            // attempting to delegate to a shim, we are more explicit about the fact that it's
            // a Notion error.
            write_str!(msg, "{} ", style("Notion error:").red().bold());
        }
    }
}

/// Formats the underlying cause of an error, if it exists
fn format_error_cause(msg: &mut String, err: &NotionError) {
    if let Some(inner) = err.as_fail().cause() {
        writeln_str!(msg);
        write_str!(
            msg,
            "{}{} ",
            style("cause").bold().underlined(),
            style(":").bold()
        );
        writeln_str!(msg, "{}", inner);
    }
}

/// Formats the backtrace for an error, if available
fn format_error_backtrace(msg: &mut String, err: &NotionError) {
    // ISSUE #75 - Once we have a way to determine backtraces without RUST_BACKTRACE, we can make this always available
    // Until then, we know that if the env var is not set, the backtrace will be empty
    if env::var("RUST_BACKTRACE").is_ok() {
        writeln_str!(msg);
        // Note: The implementation of `Display` for Backtrace includes a 'stack backtrace:' prefix
        writeln_str!(msg, "{}", err.backtrace());
    }
}

/// Determines the string to display based on the Origin of the operation.
fn action_str(origin: Origin) -> &'static str {
    match origin {
        Origin::Local => "Unpacking",
        Origin::Remote => "Fetching",
    }
}

/// Constructs a command-line progress bar based on the specified Origin enum
/// (e.g., `Origin::Remote`), details string (e.g., `"v1.23.4"`), and logical
/// length (i.e., the number of logical progress steps in the process being
/// visualized by the progress bar).
pub fn progress_bar(origin: Origin, details: &str, len: u64) -> ProgressBar {
    let display_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    let action = action_str(origin);
    let action_width = action.len() + 2; // plus 2 spaces to look nice
    let msg_width = action_width + 1 + details.len();

    //   Installing v1.23.4  [====================>                   ]  50%
    // |----------| |-----|   |--------------------------------------|  |-|
    //    action    details                      bar                 percentage
    let available_width = display_width - 2 - msg_width - 2 - 2 - 1 - 3 - 1;
    let bar_width = ::std::cmp::min(available_width, 40);

    let bar = ProgressBar::new(len);

    bar.set_message(&format!(
        "{: >width$} {}",
        style(action).green().bold(),
        details,
        width = action_width,
    ));
    bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{msg}}  [{{bar:{}.cyan/blue}}] {{percent:>3}}%",
                bar_width
            ))
            .progress_chars("=> "),
    );

    bar
}

/// Constructs a command-line progress spinner with the specified "message"
/// string. The spinner is ticked by default every 20ms.
pub fn progress_spinner(message: &str) -> ProgressBar {
    // ⠋ Fetching public registry: https://nodejs.org/dist/index.json
    let spinner = ProgressBar::new_spinner();

    spinner.set_message(message);
    spinner.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}"));
    spinner.enable_steady_tick(20); // tick the spinner every 20ms

    spinner
}
