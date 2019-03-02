//! The view layer of Notion, with utilities for styling command-line output.

use std::env;

use archive::Action;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use notion_fail::NotionError;
use term_size;

const NOTION_DEV: &'static str = "NOTION_DEV";

/// Represents the context from which an error is being reported.
pub enum ErrorContext {
    /// An error reported from the `notion` executable.
    Notion,

    /// An error reported from a shim.
    Shim,
}

/// Displays an error to stderr.
pub fn display_error(cx: ErrorContext, err: &NotionError) {
    display_error_prefix(cx);
    if err.is_user_friendly() {
        display_user_friendly_error(err);
    } else {
        display_internal_error(err);
    }
}

/// Displays a user-friendly error to stderr
fn display_user_friendly_error(err: &NotionError) {
    eprintln!("{}", err);

    if env::var(NOTION_DEV).is_ok() {
        eprintln!();
        display_development_details(err);
    }
}

/// Displays an error to stderr with a styled prefix.
fn display_error_prefix(cx: ErrorContext) {
    match cx {
        ErrorContext::Notion => {
            // Since the command here was `notion`, it would be redundant to say that this was
            // a Notion error, so we are less explicit in the heading.
            eprint!("{} ", style("error:").red().bold());
        }
        ErrorContext::Shim => {
            // Since a Notion error is rare case for a shim, it can be surprising to a user.
            // To make it extra clear that this was a failure that happened in Notion when
            // attempting to delegate to a shim, we are more explicit about the fact that it's
            // a Notion error.
            eprint!("{} ", style("Notion error:").red().bold());
        }
    }
}

/// Displays a generic message for internal errors to stderr.
fn display_internal_error(err: &NotionError) {
    eprintln!("an internal error occurred");
    eprintln!();

    if env::var(NOTION_DEV).is_ok() {
        display_development_details(err);
    } else {
        eprintln!("Notion is still a pre-alpha project, so we expect to run into some bugs,");
        eprintln!("but we'd love to hear about them so we can fix them!");
        eprintln!();
        eprintln!(
            "Please feel free to reach out to us at {} on Twitter or file an issue at:",
            style("@notionjs").cyan().bold()
        );
        eprintln!();
        eprintln!(
            "    {}",
            style("https://github.com/notion-cli/notion/issues").bold()
        );
        eprintln!();
    }
}

fn display_development_details(err: &NotionError) {
    eprintln!("{} {:?}", style("details:").yellow().bold(), err);
    eprintln!();

    // If `RUST_BACKTRACE` is set, then the backtrace will be included in the above output
    // If not, we should let the user know how to see the backtrace
    if env::var("RUST_BACKTRACE").is_err() {
        eprintln!("Run with NOTION_DEV=1 and RUST_BACKTRACE=1 for a backtrace.");
    }
}

/// Constructs a command-line progress bar with the specified Action enum
/// (e.g., `Action::Installing`), details string (e.g., `"v1.23.4"`), and logical
/// length (i.e., the number of logical progress steps in the process being
/// visualized by the progress bar).
pub fn progress_bar(action: Action, details: &str, len: u64) -> ProgressBar {
    let display_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    let msg_width = Action::MAX_WIDTH + 1 + details.len();

    //   Installing v1.23.4  [====================>                   ]  50%
    // |----------| |-----|   |--------------------------------------|  |-|
    //    action    details                      bar                 percentage
    let available_width = display_width - 2 - msg_width - 2 - 2 - 1 - 3 - 1;
    let bar_width = ::std::cmp::min(available_width, 40);

    let bar = ProgressBar::new(len);

    bar.set_message(&format!(
        "{: >width$} {}",
        style(action.to_string()).green().bold(),
        details,
        width = Action::MAX_WIDTH
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
