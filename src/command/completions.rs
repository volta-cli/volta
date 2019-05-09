use std::path::PathBuf;

use structopt::{clap::Shell, StructOpt};

use volta_core::{
    error::ErrorDetails,
    session::{ActivityKind, Session},
};
use volta_fail::{throw, ExitCode, Fallible, ResultExt};

use crate::command::Command;

#[derive(Debug, StructOpt)]
pub(crate) struct Completions {
    /// Shell to generate completions for
    #[structopt(
        takes_value = true,
        index = 1,
        raw(possible_values = "&Shell::variants()"),
        case_insensitive = true
    )]
    shell: Shell,

    /// File to write generated completions to
    #[structopt(short = "o", long = "output")]
    out_file: Option<PathBuf>,

    /// Write over an existing file, if any.
    #[structopt(short = "f", long = "force")]
    force: bool,
}

impl Command for Completions {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Completions);

        let mut app = crate::cli::Volta::clap();
        match self.out_file {
            Some(path) => {
                if path.is_file() && !self.force {
                    throw!(ErrorDetails::CompletionsOutFileError { path })
                }

                let mut file = &std::fs::File::create(&path)
                    .with_context(|_| ErrorDetails::CompletionsOutFileError { path })?;

                app.gen_completions_to("volta", self.shell, &mut file);
            }
            None => app.gen_completions_to("volta", self.shell, &mut std::io::stdout()),
        };

        session.add_event_end(ActivityKind::Completions, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
