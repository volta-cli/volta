use std::path::{Path, PathBuf};
use std::str::FromStr;

use structopt::{clap::Shell, StructOpt};

use jetson_core::{
    error::ErrorDetails,
    session::{ActivityKind, Session},
};
use jetson_fail::{throw, ExitCode, Fallible};

use crate::command::Command;

#[derive(Debug, StructOpt)]
pub(crate) struct Completions {
    /// Shell to generate completions for
    #[structopt(
        short = "s",
        long = "shell",
        raw(possible_values = "&Shell::variants()"),
        case_insensitive = true
    )]
    shell: Option<Shell>,

    /// Directory to write generated completions to
    #[structopt(short = "o", long = "out-dir")]
    out_dir: Option<PathBuf>,
}

impl Command for Completions {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Completions);

        // If the user passed a shell, we'll use that; otherwise, we'll try to
        // generate completions for their current shell. This *should* always
        // work, but if for some reason `SHELL` is unset, we handle it nicely
        // and they'll get a reasonably nice error.
        let shell = self.shell.unwrap_or(
            std::env::var_os("SHELL")
                .ok_or(ErrorDetails::UnspecifiedShell)
                .and_then(|s| {
                    Path::new(&s)
                        .components()
                        .last()
                        .ok_or(ErrorDetails::UnspecifiedShell)
                        .map(|component| component.as_os_str().to_string_lossy().into_owned())
                })
                .and_then(|shell| {
                    Shell::from_str(&shell)
                        .map_err(|_| ErrorDetails::UnrecognizedShell { name: shell })
                })?,
        );

        let mut app = crate::cli::Jetson::clap();
        match self.out_dir {
            Some(path) => {
                if path.is_dir() {
                    app.gen_completions("jetson", shell, path);
                } else {
                    throw!(ErrorDetails::CompletionsOutDirError)
                }
            }
            None => app.gen_completions_to("jetson", shell, &mut std::io::stdout()),
        }

        session.add_event_end(ActivityKind::Completions, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
