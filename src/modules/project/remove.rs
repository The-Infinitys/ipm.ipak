//! このモジュールは、プロジェクトの削除に関連する機能を提供します。
//! プロジェクトのメタデータと指定された削除モードに基づいて、プロジェクトを削除します。

use super::ExecMode;
use super::ExecShell;
use super::metadata;
use crate::utils::color::colorize::*;
use crate::utils::error::IpakError;
use std::fmt::{self, Display};

/// プロジェクト削除のオプションを定義する構造体です。
#[derive(Default)]
pub struct RemoveOptions {
    /// 削除に使用するシェル。
    pub remove_shell: ExecShell,
    /// 削除モード（例: ローカル、グローバル）。
    pub remove_mode: ExecMode,
}

impl Display for RemoveOptions {
    /// `RemoveOptions`を整形して表示します。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", "Remove Options".cyan().bold())?;
        writeln!(
            f,
            "  {}: {}",
            "remove-shell".green().bold(),
            self.remove_shell
        )?;
        writeln!(
            f,
            "  {}: {}",
            "remove-mode".green().bold(),
            self.remove_mode
        )?;
        Ok(())
    }
}

/// プロジェクトを削除します。
///
/// 指定された削除オプションに基づいて、プロジェクトディレクトリと関連ファイルを削除します。
/// 削除は`ipak/scripts/remove.sh`スクリプトを通じて実行されます。
///
/// # Arguments
/// * `opts` - 削除オプションを含む`RemoveOptions`構造体。
///
/// # Returns
/// `Ok(())` 削除が正常に完了した場合。
/// `Err(String)` 削除中にエラーが発生した場合。
pub fn remove(opts: RemoveOptions) -> Result<(), IpakError> {
    log::debug!("{}", &opts);

    let target_dir = metadata::get_dir()?;

    let project_metadata = metadata::metadata()?;

    let mut remove_process = opts.remove_shell.generate();

    remove_process
        .current_dir(&target_dir)
        .env("IPAK_PROJECT_NAME", &project_metadata.about.package.name)
        .env(
            "IPAK_PROJECT_VERSION",
            project_metadata.about.package.version.to_string(),
        )
        .env("IPAK_REMOVE_MODE", opts.remove_mode.to_string())
        .arg("ipak/scripts/remove.sh");

    let status = remove_process.status()?;

    if status.success() {
        Ok(())
    } else {
        Err(IpakError::CommandExecution(status.code().unwrap_or(-1)))
    }
}
