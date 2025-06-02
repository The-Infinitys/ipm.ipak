use super::ExecMode;
use super::ExecShell;
use super::metadata::{self, metadata};
use crate::dprintln;
use crate::modules::version::Version;
use crate::utils::color::colorize::*;
use std::fmt::{self, Display};
use std::process::Command;
#[derive(Default)]
pub struct InstallOptions {
    pub install_shell: ExecShell,
    pub install_mode: ExecMode,
}
impl Display for InstallOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = [
            format!("{}{}", "Install Options".cyan().bold(), ":"),
            format!(
                "  {}{} {}",
                "install-shell".green().bold(),
                ":",
                self.install_shell
            ),
            format!(
                "  {}{} {}",
                "install-mode".green().bold(),
                ":",
                self.install_mode
            ),
        ];
        for line in lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}
pub fn install(opts: InstallOptions) -> Result<(), String> {
    dprintln!("{}", &opts);
    let target_dir = metadata::get_dir();
    let target_dir = match target_dir {
        Ok(path) => path,
        Err(e) => {
            let msg = format!("Error: {}", e);
            eprintln!("{}", msg);
            return Err(msg);
        }
    };
    let project_metadata = metadata().unwrap();

    // Configure install shell
    fn setup_execshell(
        cmd: &mut Command,
        target_dir: &std::path::Path,
        project_name: &str,
        project_version: &Version,
        install_mode: &ExecMode,
    ) {
        cmd.current_dir(target_dir)
            .env("IPAK_PROJECT_NAME", project_name)
            .env("IPAK_PROJECT_VERSION", project_version.to_string())
            .env("IPAK_INSTALL_MODE", install_mode.to_string())
            .arg("ipak/scripts/install.sh");
    }

    let mut install_process = opts.install_shell.generate();
    setup_execshell(
        &mut install_process,
        &target_dir,
        &project_metadata.about.package.name,
        &project_metadata.about.package.version,
        &opts.install_mode,
    );

    // Execute the install process and handle the result
    let status = install_process.status().map_err(|e| {
        format!("Failed to execute install process: {}", e)
    })?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Install process failed with status: {}", status))
    }
}
