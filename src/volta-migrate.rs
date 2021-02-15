use structopt::StructOpt;

use volta_core::error::{report_error, ExitCode};
use volta_core::layout::volta_home;
use volta_core::log::{LogContext, LogVerbosity, Logger};
use volta_migrate::run_migration;

#[derive(StructOpt)]
#[structopt(
    name = "volta-migrate",
    about = "Migrates the Volta directory to the latest version",
    raw(global_setting = "structopt::clap::AppSettings::ColoredHelp"),
    raw(global_setting = "structopt::clap::AppSettings::ColorAuto"),
    raw(global_setting = "structopt::clap::AppSettings::DeriveDisplayOrder"),
    raw(global_setting = "structopt::clap::AppSettings::DisableVersion")
)]
struct VoltaMigrate {
    #[structopt(
        long = "no-create",
        help = "Runs migration only if the Volta directory already exists",
        global = true
    )]
    pub(crate) no_create: bool,
}

pub fn main() {
    let volta_migrate = VoltaMigrate::from_args();

    Logger::init(LogContext::Migration, LogVerbosity::Default)
        .expect("Only a single Logger should be initialized");

    if volta_migrate.no_create && !volta_home().map_or(false, |home| home.root().exists()) {
        ExitCode::Success.exit();
    }

    let exit_code = match run_migration() {
        Ok(()) => ExitCode::Success,
        Err(err) => {
            report_error(env!("CARGO_PKG_VERSION"), &err);
            err.exit_code()
        }
    };

    exit_code.exit();
}
