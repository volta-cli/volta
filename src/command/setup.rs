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
    use std::env;
    use std::fs::{File, OpenOptions};
    use std::io::{self, Read, Write};
    use std::path::Path;

    use log::debug;
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

    enum ProfileState {
        NotFound,
        FoundMentionsVolta,
        FoundWithoutVolta(File),
    }

    pub fn setup_environment() -> Fallible<()> {
        let user_home_dir = dirs::home_dir().ok_or(ErrorDetails::NoHomeEnvironmentVar)?;
        let home = volta_home()?;

        debug!("Searching for profiles to update");
        let env_profile = env::var("PROFILE");

        let found_profile = PROFILES
            .iter()
            .chain(&env_profile.as_ref().map(String::as_str))
            .fold(false, |prev, path| {
                let profile = user_home_dir.join(path);
                match check_profile(&profile) {
                    ProfileState::NotFound => {
                        debug!("Profile script not found: {}", profile.display());
                        prev
                    }
                    ProfileState::FoundMentionsVolta => {
                        debug!(
                            "Profile script found, already mentions Volta: {}",
                            profile.display()
                        );
                        true
                    }
                    ProfileState::FoundWithoutVolta(file) => {
                        debug!("Profile script found: {}", profile.display());
                        let result = match profile.extension() {
                            Some(ext) if ext == "fish" => modify_profile_fish(file, home.root()),
                            _ => modify_profile_sh(file, home.root()),
                        };

                        match result {
                            Ok(()) => true,
                            Err(err) => {
                                debug!("Could not modify profile script: {}", err);
                                prev
                            }
                        }
                    }
                }
            });

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

    fn check_profile(profile: &Path) -> ProfileState {
        match open_for_read_write(profile) {
            Ok(mut file) => {
                let mut contents = String::new();
                match file.read_to_string(&mut contents) {
                    Ok(_) => {
                        if contents.contains("VOLTA_HOME") {
                            ProfileState::FoundMentionsVolta
                        } else {
                            ProfileState::FoundWithoutVolta(file)
                        }
                    }
                    Err(_) => ProfileState::NotFound,
                }
            }
            Err(_) => ProfileState::NotFound,
        }
    }

    fn modify_profile_sh(mut file: File, volta_home: &Path) -> io::Result<()> {
        write!(
            file,
            "\nexport VOLTA_HOME=\"{}\"\ngrep --silent \"$VOLTA_HOME/bin\" <<< $PATH || export PATH=\"$VOLTA_HOME/bin:$PATH\"\n",
            volta_home.display()
        )
    }

    fn modify_profile_fish(mut file: File, volta_home: &Path) -> io::Result<()> {
        write!(
            file,
            "\nset -gx VOLTA_HOME \"{}\"\nstring match -r \".volta\" \"$PATH\" > /dev/null; or set -gx PATH \"$VOLTA_HOME/bin\" $PATH\n",
            volta_home.display()
        )
    }

    fn open_for_read_write<P: AsRef<Path>>(path: P) -> io::Result<File> {
        OpenOptions::new().read(true).write(true).open(path)
    }
}

#[cfg(windows)]
mod os {
    use std::process::Command;

    use log::debug;
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

            debug!("Modifying User Path with command: {:?}", command);
            let output = command
                .output()
                .with_context(|_| ErrorDetails::WriteUserPathError)?;

            if !output.status.success() {
                debug!("[setx stderr]\n{}", String::from_utf8_lossy(&output.stderr));
                debug!("[setx stdout]\n{}", String::from_utf8_lossy(&output.stdout));
                return Err(ErrorDetails::WriteUserPathError.into());
            }
        }

        Ok(())
    }
}