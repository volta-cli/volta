use super::Package;
use crate::command::create_command;
use crate::error::{Context, ErrorKind, Fallible};
use crate::platform::Image;
use crate::style::progress_spinner;
use log::debug;

impl Package {
    /// Use `npm install --global` to install the package
    ///
    /// Sets the environment variable `npm_config_prefix` to redirect the install to the Volta
    /// data directory, taking advantage of the standard global install behavior with a custom
    /// location
    pub fn global_install(&self, platform_image: &Image) -> Fallible<()> {
        let package = self.to_string();
        let mut command = create_command("npm");
        command.args(&[
            "install",
            "--global",
            "--loglevel=warn",
            "--no-update-notifier",
            "--no-audit",
        ]);
        command.arg(&package);
        command.env("PATH", platform_image.path()?);
        command.env("npm_config_prefix", self.staging.path());

        debug!("Installing {} with command: {:?}", package, command);
        let spinner = progress_spinner(&format!("Installing {}", package));
        let output_result = command
            .output()
            .with_context(|| ErrorKind::PackageInstallFailed {
                package: package.clone(),
            });
        spinner.finish_and_clear();
        let output = output_result?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        debug!("[install stderr]\n{}", stderr);
        debug!(
            "[install stdout]\n{}",
            String::from_utf8_lossy(&output.stdout)
        );

        if output.status.success() {
            Ok(())
        } else if stderr.contains("code E404") {
            // npm outputs "code E404" as part of the error output when a package couldn't be found
            // Detect that and show a nicer error message (since we likely know the problem in that case)
            Err(ErrorKind::PackageNotFound { package }.into())
        } else {
            Err(ErrorKind::PackageInstallFailed { package }.into())
        }
    }
}
