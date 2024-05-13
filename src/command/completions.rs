use std::path::PathBuf;

use clap::CommandFactory;
use clap_complete::Shell;
use log::info;

use volta_core::{
    error::{Context, ErrorKind, ExitCode, Fallible},
    session::{ActivityKind, Session},
    style::{note_prefix, success_prefix},
};

use crate::command::Command;

#[derive(Debug, clap::Args)]
pub(crate) struct Completions {
    /// Shell to generate completions for
    #[arg(index = 1, ignore_case = true, required = true)]
    shell: Shell,

    /// File to write generated completions to
    #[arg(short, long = "output")]
    out_file: Option<PathBuf>,

    /// Write over an existing file, if any.
    #[arg(short, long)]
    force: bool,
}

impl Command for Completions {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Completions);

        let mut app = crate::cli::Volta::command();
        let app_name = app.get_name().to_owned();
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

                clap_complete::generate(self.shell, &mut app, app_name, &mut file);

                info!(
                    "{} installed completions to {}",
                    success_prefix(),
                    path.display()
                );
            }
            None => clap_complete::generate(self.shell, &mut app, app_name, &mut std::io::stdout()),
        };

        session.add_event_end(ActivityKind::Completions, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
