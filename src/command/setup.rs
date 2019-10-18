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

        // TODO - CPIERCE: Show spinner and messages for each step (similar to current shell installer)

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
            // TODO - CPIERCE: Make debug statements about what is happening
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

                // TODO - CPIERCE: On error, show error details in debug output
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
    use std::process::Command;
    use volta_core::error::ErrorDetails;
    use volta_core::layout::volta_home;
    use volta_fail::{Fallible, ResultExt};
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    pub fn setup_environment() -> Fallible<()> {
        let shim_dir = volta_home()?.shim_dir().to_string_lossy().to_string();
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu
            .open_subkey("Environment")
            .with_context(|_| ErrorDetails::ReadUserPathError)?;
        let path: String = env
            .get_value("Path")
            .with_context(|_| ErrorDetails::ReadUserPathError)?;

        if !path.contains(&shim_dir) {
            // Use `setx` command to edit the user Path environment variable
            let mut command = Command::new("setx");
            command.arg("Path");
            command.arg(format!("{};{}", shim_dir, path));

            // TODO - CPIERCE: Debug show the command being run
            let output = command
                .output()
                .with_context(|_| ErrorDetails::WriteUserPathError)?;

            if !output.status.success() {
                // TODO - CPIERCE: If this fails, write the stdout and stderr to the debug log
                return Err(ErrorDetails::WriteUserPathError.into());
            }
        }

        Ok(())
    }
}
