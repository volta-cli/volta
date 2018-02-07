use std::convert::Into;

use console::style;
use failure;
use indicatif::{ProgressBar, ProgressStyle};
use term_size;

pub fn display_error<E: Into<failure::Error>>(err: E) {
    display_error_prefix();
    eprintln!("{}", err.into());
}

pub fn display_error_prefix() {
    eprint!("{} ", style("error:").red().bold());
}

pub fn progress_bar(msg: &str, len: u64) -> ProgressBar {
    let display_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    let msg_width = msg.len();

    // Installing v1.23.4  [====================>                   ]  50%
    // |----------------|   |--------------------------------------|  |-|
    //         msg                           bar                   percentage
    let available_width = display_width - 2 - msg_width - 2 - 2 - 1 - 3 - 1;
    let bar_width = ::std::cmp::min(available_width, 40);

    let bar = ProgressBar::new(len);

    bar.set_message(msg);
    bar.set_style(ProgressStyle::default_bar()
        // FIXME: instead of fixed 40 compute based on console size
        .template(&format!("{{msg}}  [{{bar:{}.cyan/blue}}] {{percent:>3}}%", bar_width))
        .progress_chars("=> "));

    bar
}
