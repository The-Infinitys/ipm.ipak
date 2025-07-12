//! このモジュールは、プロジェクトの完全な削除（パージ）に関連する機能を提供します。
//! プロジェクトのメタデータと指定されたパージモードに基づいて、プロジェクトを完全に削除します。

use super::ExecMode;
use super::ExecShell;
use super::metadata;
use crate::dprintln;
use crate::utils::color::colorize::*;
use std::fmt::{self, Display};

/// プロジェクトパージのオプションを定義する構造体です。
#[derive(Default)]
pub struct PurgeOptions {
    /// パージに使用するシェル。
    pub purge_shell: ExecShell,
    /// パージモード（例: ローカル、グローバル）。
    pub purge_mode: ExecMode,
}

impl Display for PurgeOptions {
    /// `PurgeOptions`を整形して表示します。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", "Purge Options".cyan().bold())?;
        writeln!(
            f,
            "  {}: {}",
            "purge-shell".green().bold(),
            self.purge_shell
        )?;
        writeln!(
            f,
            "  {}: {}",
            "purge-mode".green().bold(),
            self.purge_mode
        )?;
        Ok(())
    }
}

/// プロジェクトを完全に削除（パージ）します。
///
/// 指定されたパージオプションに基づいて、プロジェクトディレクトリと関連ファイルを完全に削除します。
/// パージは`ipak/scripts/purge.sh`スクリプトを通じて実行されます。
///
/// # Arguments
/// * `opts` - パージオプションを含む`PurgeOptions`構造体。
///
/// # Returns
/// `Ok(())` パージが正常に完了した場合。
/// `Err(String)` パージ中にエラーが発生した場合。
pub fn purge(opts: PurgeOptions) -> Result<(), String> {
    log::debug!("{}", &opts);

    let target_dir = metadata::get_dir().map_err(|_| {
        "Error: Couldn't find Ipak Directory. Make sure you are in an ipak project.".to_string()
    })?;

    let project_metadata = metadata::metadata().map_err(|e| {
        format!("Error: Failed to read project metadata: {:?}", e)
    })?;

    let mut purge_process = opts.purge_shell.generate();
    purge_process
        .current_dir(&target_dir)
        .env("IPAK_PROJECT_NAME", &project_metadata.about.package.name)
        .env(
            "IPAK_PROJECT_VERSION",
            project_metadata.about.package.version.to_string(),
        )
        .env("IPAK_PURGE_MODE", opts.purge_mode.to_string())
        .arg("ipak/scripts/purge.sh");

    let status = purge_process
        .status()
        .map_err(|e| format!("Failed to execute purge process: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        let code_info = status
            .code()
            .map_or("".to_string(), |c| format!(" (exit code: {})", c));
        Err(format!(
            "Purge process failed with status: {}{}",
            status, code_info
        ))
    }
}
