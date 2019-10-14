use log::info;
use structopt::StructOpt;
use volta_core::layout::bootstrap_volta_dirs;
use volta_core::session::{ActivityKind, Session};
use volta_core::style::success_prefix;
use volta_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Setup {}

impl Command for Setup {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Setup);

        // ISSUE #566 - Once we have a working migration, we can leave the creation of the
        // directory structure to the migration and not have to call it here
        bootstrap_volta_dirs()?;
        os::setup_environment()?;

        info!(
            "{} Setup complete. Open a new terminal to start using Volta!",
            success_prefix()
        );

        session.add_event_end(ActivityKind::Setup, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}

#[cfg(unix)]
mod os {
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    use std::path::Path;

    use volta_core::error::ErrorDetails;
    use volta_core::layout::volta_home;
    use volta_fail::Fallible;

    const PROFILES: [&'static str; 5] = [
        ".profile",
        ".bash_profile",
        ".bashrc",
        ".zshrc",
        ".config/fish/config.fish",
    ];

    pub fn setup_environment() -> Fallible<()> {
        let user_home_dir = dirs::home_dir().ok_or(ErrorDetails::NoHomeEnvironmentVar)?;
        let home = volta_home()?;
        let mut found_profile = false;

        for path in PROFILES.iter() {
            let profile = user_home_dir.join(path);
            if let Some(mut file) = open_for_append(&profile) {
                let result = match profile.extension() {
                    Some(ext) if ext == "fish" => write!(
                        file,
                        "\nset -gx VOLTA_HOME \"{}\"\nstring match -r \".volta\" \"$PATH\" > /dev/null; or set -gx PATH \"$VOLTA_HOME/bin\" $PATH\n",
                        home.root().display()
                    ),
                    _ => write!(
                        file,
                        "\nexport VOLTA_HOME=\"{}\"\ngrep --silent \"$VOLTA_HOME/bin\" <<< $PATH || export PATH=\"$VOLTA_HOME/bin:$PATH\"\n",
                        home.root().display()
                    ),
                };

                if result.is_ok() {
                    found_profile = true;
                }
            }
        }

        if found_profile {
            Ok(())
        } else {
            Err(ErrorDetails::NoShellProfile {
                env_profile: String::new(),
                bin_dir: home.shim_dir().to_owned(),
            }
            .into())
        }
    }

    fn open_for_append<P: AsRef<Path>>(path: P) -> Option<File> {
        OpenOptions::new().append(true).open(path).ok()
    }
}

#[cfg(windows)]
mod os {
    use volta_fail::Fallible;
    pub fn setup_environment() -> Fallible<()> {
        // In windows, need to edit HKEY_CURRENT_USER\Environment to modify the User PATH
        unimplemented!();
    }
}
