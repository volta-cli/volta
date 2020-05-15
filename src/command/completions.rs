use std::path::PathBuf;

use log::info;
use structopt::{clap::Shell, StructOpt};

use volta_core::{
    error::{Context, ErrorKind, ExitCode, Fallible},
    session::{ActivityKind, Session},
    style::{note_prefix, success_prefix},
};

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
                    return Err(ErrorKind::CompletionsOutFileError { path }.into());
                }

                // The user may have passed a path that does not yet exist. If
                // so, we create it, informing the user we have done so.
                if let Some(parent) = path.parent() {
                    if !parent.is_dir() {
                        info!(
                            "{} {} does not exist, creating it",
                            note_prefix(),
                            parent.display()
                        );
                        std::fs::create_dir_all(parent).with_context(|| {
                            ErrorKind::CreateDirError {
                                dir: parent.to_path_buf(),
                            }
                        })?;
                    }
                }

                let mut file = &std::fs::File::create(&path).with_context(|| {
                    ErrorKind::CompletionsOutFileError {
                        path: path.to_path_buf(),
                    }
                })?;

                app.gen_completions_to("volta", self.shell, &mut file);

                info!(
                    "{} installed completions to {}",
                    success_prefix(),
                    path.display()
                );
            }
            None => app.gen_completions_to("volta", self.shell, &mut std::io::stdout()),
        };

        session.add_event_end(ActivityKind::Completions, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
