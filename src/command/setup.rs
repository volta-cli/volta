use log::info;
use structopt::StructOpt;
use volta_core::layout::volta_home;
use volta_core::session::{ActivityKind, Session};
use volta_core::shim::regenerate_shims_for_dir;
use volta_core::style::success_prefix;
use volta_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Setup {}

impl Command for Setup {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Setup);

        os::setup_environment()?;
        regenerate_shims_for_dir(volta_home()?.shim_dir())?;

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
    use std::fs::File;
    use std::io::{self, BufRead, BufReader, Write};
    use std::path::Path;

    use log::{debug, warn};
    use volta_core::error::ErrorDetails;
    use volta_core::layout::volta_home;
    use volta_fail::Fallible;

    const PROFILES: [&str; 5] = [
        ".profile",
        ".bash_profile",
        ".bashrc",
        ".zshrc",
        ".config/fish/config.fish",
    ];

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
                match read_profile_without_volta(&profile) {
                    Some(contents) => {
                        debug!("Profile script found: {}", profile.display());

                        let write_profile = match profile.extension() {
                            Some(ext) if ext == "fish" => write_profile_fish,
                            _ => write_profile_sh,
                        };

                        match write_profile(&profile, contents, home.root()) {
                            Ok(()) => true,
                            Err(err) => {
                                warn!(
                                    "Found profile script, but could not modify it: {}",
                                    profile.display()
                                );
                                debug!("Profile modification error: {}", err);
                                prev
                            }
                        }
                    }
                    None => {
                        debug!("Profile script not found: {}", profile.display());
                        prev
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

    fn read_profile_without_volta(path: &Path) -> Option<String> {
        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);

        reader
            .lines()
            .filter(|line_result| match line_result {
                Ok(line) if !line.contains("VOLTA") => true,
                Ok(_) => false,
                Err(_) => true,
            })
            .collect::<io::Result<Vec<String>>>()
            .map(|lines| lines.join("\n"))
            .ok()
    }

    fn write_profile_sh(path: &Path, contents: String, volta_home: &Path) -> io::Result<()> {
        let mut file = File::create(path)?;
        write!(
            file,
            "{}\nexport VOLTA_HOME=\"{}\"\ngrep --silent \"$VOLTA_HOME/bin\" <<< $PATH || export PATH=\"$VOLTA_HOME/bin:$PATH\"\n",
            contents,
            volta_home.display(),
        )
    }

    fn write_profile_fish(path: &Path, contents: String, volta_home: &Path) -> io::Result<()> {
        let mut file = File::create(path)?;
        write!(
            file,
            "{}\nset -gx VOLTA_HOME \"{}\"\nstring match -r \".volta\" \"$PATH\" > /dev/null; or set -gx PATH \"$VOLTA_HOME/bin\" $PATH\n",
            contents,
            volta_home.display(),
        )
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
