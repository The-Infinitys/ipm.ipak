//! このモジュールは、`ipak`プロジェクトの設定に関連する機能を提供します。

use super::ExecMode;
use super::ExecShell;
use crate::utils::error::IpakError;

/// プロジェクト設定のオプションを定義する構造体です。
#[derive(Debug, Clone)]
pub struct ConfigureOptions {
    /// 設定モード（ローカルまたはグローバル）。
    pub configure_mode: ExecMode,
    /// 設定に使用するシェル。
    pub configure_shell: ExecShell,
}

/// プロジェクトを設定します。
///
/// # Arguments
/// * `opts` - 設定オプション。
///
/// # Returns
/// `Ok(())` 設定が正常に完了した場合。
/// `Err(IpakError)` 設定中にエラーが発生した場合。
pub fn configure(opts: ConfigureOptions) -> Result<(), IpakError> {
    log::debug!(
        "Configuring project in {:?} mode using {:?} shell",
        opts.configure_mode,
        opts.configure_shell
    );

    let mut command = opts.configure_shell.generate();
    command.arg("ipak/scripts/configure.sh");

    let status = command.status()?;

    if status.success() {
        log::debug!("Project configured successfully.");
        Ok(())
    } else {
        Err(IpakError::from(std::io::Error::other(
            format!("Failed to configure project: {:?}", status.code()),
        )))
    }
}
