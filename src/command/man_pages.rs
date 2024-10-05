use std::path::PathBuf;

use clap::CommandFactory;
use clap_mangen::Man;
use log::info;

use volta_core::{
    error::{Context, ErrorKind, ExitCode, Fallible},
    session::{ActivityKind, Session},
    style::{note_prefix, success_prefix},
};

use crate::command::Command;

#[derive(Debug, clap::Args)]
pub(crate) struct ManPages {
    /// File to write generated man pages to
    #[arg(short, long = "output")]
    out_file: Option<PathBuf>,

    /// Write over an existing file, if any.
    #[arg(short, long)]
    force: bool,
}

impl Command for ManPages {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::ManPages);

        let app = crate::cli::Volta::command();
        let man = Man::new(app.clone());

        match self.out_file {
            Some(path) => {
                if path.is_file() && !self.force {
                    return Err(ErrorKind::ManPagesOutFileError { path }.into());
                }

                // Create parent directory if it doesn't exist
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

                let mut file = std::fs::File::create(&path).with_context(|| {
                    ErrorKind::ManPagesOutFileError {
                        path: path.to_path_buf(),
                    }
                })?;

                man.render(&mut file)
                    .map_err(|_e| ErrorKind::ManPagesOutFileError {
                        path: path.to_path_buf(),
                    })?;

                info!(
                    "{} generated man pages to {}",
                    success_prefix(),
                    path.display()
                );
            }
            None => {
                man.render(&mut std::io::stdout()).map_err(|_e| {
                    ErrorKind::ManPagesOutFileError {
                        path: PathBuf::from("stdout"),
                    }
                })?;
            }
        };

        session.add_event_end(ActivityKind::ManPages, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
