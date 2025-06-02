use super::ExecShell;
use super::metadata::{self, metadata};
use crate::dprintln;
use crate::modules::version::Version;
use crate::utils::color::colorize::*;
use std::fmt::{self, Display};
use std::process::Command;

#[derive(Default)]
pub struct BuildOptions {
    pub build_mode: BuildMode,
    pub build_shell: ExecShell,
}

impl Display for BuildOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = [
            format!("{}{}", "Build Options".cyan().bold(), ":"),
            format!(
                "  {}{} {}",
                "build-mode".green().bold(),
                ":",
                self.build_mode
            ),
            format!(
                "  {}{} {}",
                "build-shell".green().bold(),
                ":",
                self.build_shell
            ),
        ];
        for line in lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub enum BuildMode {
    Release,
    #[default]
    Debug,
}

impl Display for BuildMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildMode::Release => {
                write!(f, "release")
            }
            BuildMode::Debug => {
                write!(f, "debug")
            }
        }
    }
}

pub fn build(opts: BuildOptions) -> Result<(), String> {
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

    // Configure build shell
    fn setup_execshell(
        cmd: &mut Command,
        target_dir: &std::path::Path,
        project_name: &str,
        project_version: &Version,
        build_mode: &BuildMode,
    ) {
        let build_mode = build_mode.to_string();
        cmd.current_dir(target_dir)
            .env("IPAK_PROJECT_NAME", project_name)
            .env("IPAK_PROJECT_VERSION", project_version.to_string())
            .env("IPAK_BUILD_MODE", build_mode)
            .arg("ipak/scripts/build.sh");
    }

    let mut build_process = opts.build_shell.generate();
    setup_execshell(
        &mut build_process,
        &target_dir,
        &project_metadata.about.package.name,
        &project_metadata.about.package.version,
        &opts.build_mode,
    );

    // Execute the build process and handle the result
    let status = build_process
        .status()
        .map_err(|e| format!("Failed to execute build process: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Build process failed with status: {}", status))
    }
}
