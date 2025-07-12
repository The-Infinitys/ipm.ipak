//! このモジュールは、プロジェクト内で定義されたスクリプトを実行する機能を提供します。
//! プロジェクトのメタデータに基づいて、指定されたシェルでスクリプトを実行します。

use super::metadata::{self, metadata};
use crate::modules::project::ExecShell;
use crate::utils::color::colorize::*;
use crate::utils::version::Version;
use std::process::Command;

/// プロジェクト内で指定されたスクリプトを実行します。
///
/// プロジェクトのメタデータから実行ディレクトリとプロジェクト名、バージョンを取得し、
/// 指定されたシェルでスクリプトを実行します。スクリプトは`ipak/scripts/{name}.sh`に存在すると仮定されます。
///
/// # Arguments
/// * `shell` - スクリプトの実行に使用するシェル。`None`の場合はデフォルトのシェルが使用されます。
/// * `name` - 実行するスクリプトの名前（例: "build", "install"）。
/// * `args` - スクリプトに渡す追加の引数。
///
/// # Returns
/// `Ok(())` スクリプトが正常に実行された場合。
/// `Err(String)` スクリプトの実行に失敗した場合、または必要なメタデータが見つからない場合。
pub fn run(
    shell: Option<ExecShell>,
    name: &str,
    args: Vec<String>,
) -> Result<(), String> {
    let name = name.to_ascii_lowercase();
    log::info!("{}: {}", "Run".bold().green(), name.bold().cyan());
    let exec_shell = shell.unwrap_or_default();
    let target_dir =
        metadata::get_dir().map_err(|e| format!("Error: {}", e))?;
    let project_metadata =
        metadata().map_err(|e| format!("Error: {}", e))?;

    fn setup_execshell(
        cmd: &mut Command,
        name: &str,
        target_dir: &std::path::Path,
        project_name: &str,
        project_version: &Version,
        args: Vec<String>,
    ) {
        cmd.current_dir(target_dir)
            .env("IPAK_PROJECT_NAME", project_name)
            .env("IPAK_PROJECT_VERSION", project_version.to_string())
            .arg(format!("ipak/scripts/{}.sh", name))
            .arg("--")
            .args(args);
    }

    let mut exec_process = exec_shell.generate();
    setup_execshell(
        &mut exec_process,
        &name,
        &target_dir,
        &project_metadata.about.package.name,
        &project_metadata.about.package.version,
        args,
    );

    let status = exec_process
        .status()
        .map_err(|e| format!("Failed to execute exec process: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("exec process failed with status: {}", status))
    }
}
