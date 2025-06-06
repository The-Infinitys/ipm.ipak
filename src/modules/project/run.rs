use super::metadata::{self, metadata};
use crate::modules::project::ExecShell;
use crate::modules::version::Version;
use crate::utils::color::colorize::*;
use cmd_arg::cmd_arg;
use std::process::Command;
use std::str::FromStr;

pub fn run(name: &str, args: Vec<&cmd_arg::Option>) -> Result<(), String> {
    let name = name.to_ascii_lowercase();
    println!("{}: {}", "Run".bold().green(), name.bold().cyan());
    let mut exec_shell: ExecShell = ExecShell::default();
    for arg in args {
        match arg.opt_str.as_str() {
            "--shell" => {
                if !arg.opt_values.is_empty() {
                    exec_shell = ExecShell::from_str(
                        arg.opt_values.first().unwrap(),
                    )?;
                }
            }
            _ => continue,
        }
    }
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

    // Configure exec shell
    fn setup_execshell(
        cmd: &mut Command,
        name: &str,
        target_dir: &std::path::Path,
        project_name: &str,
        project_version: &Version,
    ) {
        cmd.current_dir(target_dir)
            .env("IPAK_PROJECT_NAME", project_name)
            .env("IPAK_PROJECT_VERSION", project_version.to_string())
            .arg(format!("ipak/scripts/{}.sh", name));
    }

    let mut exec_process = exec_shell.generate();
    setup_execshell(
        &mut exec_process,
        &name,
        &target_dir,
        &project_metadata.about.package.name,
        &project_metadata.about.package.version,
    );

    // Execute the exec process and handle the result
    let status = exec_process
        .status()
        .map_err(|e| format!("Failed to execute exec process: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("exec process failed with status: {}", status))
    }
}
