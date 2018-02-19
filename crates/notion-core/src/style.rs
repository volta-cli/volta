//! The view layer of Notion, with utilities for styling command-line output.

use std::convert::Into;

use console::style;
use failure;
use indicatif::{ProgressBar, ProgressStyle};
use term_size;

/// Displays an error to stderr.
pub fn display_error<E: Into<failure::Error>>(err: E) {
    display_error_prefix();
    eprintln!("{}", err.into());
}

/// Displays an error to stderr with a styled `"error:"` prefix.
pub fn display_error_prefix() {
    eprint!("{} ", style("error:").red().bold());
}

/// Constructs a command-line progress bar with the specified "action" string
/// (e.g., `"Installing"`), details string (e.g., `"v1.23.4"`), and logical
/// length (i.e., the number of logical progress steps in the process being
/// visualized by the progress bar).
pub fn progress_bar(action: &str, details: &str, len: u64) -> ProgressBar {
    let display_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    let msg_width = 12 + 1 + details.len();

    //   Installing v1.23.4  [====================>                   ]  50%
    // |----------| |-----|   |--------------------------------------|  |-|
    //    action    details                      bar                 percentage
    let available_width = display_width - 2 - msg_width - 2 - 2 - 1 - 3 - 1;
    let bar_width = ::std::cmp::min(available_width, 40);

    let bar = ProgressBar::new(len);

    bar.set_message(&format!("{: >12} {}", style(action).green().bold(), details));
    bar.set_style(ProgressStyle::default_bar()
        // ISSUE (#35): instead of fixed 40 compute based on console size
        .template(&format!("{{msg}}  [{{bar:{}.cyan/blue}}] {{percent:>3}}%", bar_width))
        .progress_chars("=> "));

    bar
}
