use std::process::{Command, ExitStatus};

use volta_core::error::{Context, ErrorKind, VoltaError};
use volta_core::layout::{volta_home, volta_install};

pub enum Error {
    Volta(VoltaError),
    Tool(i32),
}

pub fn ensure_layout() -> Result<(), Error> {
    let home = volta_home().map_err(Error::Volta)?;

    if !home.layout_file().exists() {
        let install = volta_install().map_err(Error::Volta)?;
        Command::new(install.migrate_executable())
            .env("VOLTA_LOGLEVEL", format!("{}", log::max_level()))
            .status()
            .with_context(|| ErrorKind::CouldNotStartMigration)
            .into_result()?;
    }

    Ok(())
}

pub trait IntoResult<T> {
    fn into_result(self) -> Result<T, Error>;
}

impl IntoResult<()> for Result<ExitStatus, VoltaError> {
    fn into_result(self) -> Result<(), Error> {
        match self {
            Ok(status) => {
                if status.success() {
                    Ok(())
                } else {
                    let code = status.code().unwrap_or(1);
                    Err(Error::Tool(code))
                }
            }
            Err(err) => Err(Error::Volta(err)),
        }
    }
}
