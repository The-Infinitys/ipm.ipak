//! このモジュールは、プロジェクトのインストールに関連する機能を提供します。
//! プロジェクトのメタデータと指定されたインストールモードに基づいて、プロジェクトをインストールします。

use super::ExecMode;
use super::ExecShell;
use super::metadata::{self, metadata};
use crate::dprintln;
use crate::utils::color::colorize::*;
use crate::utils::version::Version;
use std::fmt::{self, Display};
use std::process::Command;

/// プロジェクトインストールのオプションを定義する構造体です。
#[derive(Default)]
pub struct InstallOptions {
    /// インストールに使用するシェル。
    pub install_shell: ExecShell,
    /// インストールモード（例: ローカル、グローバル）。
    pub install_mode: ExecMode,
}

impl Display for InstallOptions {
    /// `InstallOptions`を整形して表示します。
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

/// プロジェクトをインストールします。
///
/// 指定されたインストールオプションに基づいて、プロジェクトをインストールします。
/// インストールは`ipak/scripts/install.sh`スクリプトを通じて実行されます。
///
/// # Arguments
/// * `opts` - インストールオプションを含む`InstallOptions`構造体。
///
/// # Returns
/// `Ok(())` インストールが正常に完了した場合。
/// `Err(String)` インストール中にエラーが発生した場合。
pub fn install(opts: InstallOptions) -> Result<(), String> {
    log::debug!("{}", &opts);
    let target_dir =
        metadata::get_dir().map_err(|e| format!("Error: {}", e))?;
    let project_metadata =
        metadata().map_err(|e| format!("Error: {}", e))?;

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

    let status = install_process.status().map_err(|e| {
        format!("Failed to execute install process: {}", e)
    })?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Install process failed with status: {}", status))
    }
}
