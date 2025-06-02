use super::ExecMode;
use super::ExecShell;
use super::metadata; // metadata モジュール全体をインポート
use crate::dprintln;
use crate::utils::color::colorize::*;
use std::fmt::{self, Display};
/// パッケージのパージ（完全削除）に関するオプションを保持する構造体。
#[derive(Default)]
pub struct PurgeOptions {
    /// パージ処理を実行するシェル。
    pub purge_shell: ExecShell,
    pub purge_mode: ExecMode,
}

impl Display for PurgeOptions {
    /// PurgeOptions の内容を整形して表示します。
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

/// プロジェクトのパージ処理を実行します。
///
/// この関数は、プロジェクトのメタデータを読み込み、設定されたシェルを使用して
/// パージスクリプト（`ipak/scripts/purge.sh`）を実行します。
/// パージ処理が成功した場合は `Ok(())` を返し、失敗した場合はエラーメッセージを含む `Err(String)` を返します。
///
/// # 引数
///
/// * `opts`: パージ処理に関するオプション (`PurgeOptions`)。
///
/// # 戻り値
///
/// パージ処理の成否を示す `Result<(), String>`。
pub fn purge(opts: PurgeOptions) -> Result<(), String> {
    dprintln!("{}", &opts); // デバッグ情報を出力

    // プロジェクトのメタデータディレクトリを取得
    let target_dir = metadata::get_dir().map_err(|_| {
        "Error: Couldn't find Ipak Directory. Make sure you are in an ipak project.".to_string()
    })?;

    // プロジェクトのメタデータを読み込む
    let project_metadata = metadata::metadata().map_err(|e| {
        format!("Error: Failed to read project metadata: {:?}", e)
    })?;

    // シェルコマンドの準備
    let mut purge_process = opts.purge_shell.generate();
    purge_process
        .current_dir(&target_dir) // プロジェクトディレクトリを作業ディレクトリに設定
        .env("IPAK_PROJECT_NAME", &project_metadata.about.package.name) // パッケージ名を環境変数に設定
        .env(
            "IPAK_PROJECT_VERSION",
            project_metadata.about.package.version.to_string(),
        )
        .env("IPAK_PURGE_MODE", opts.purge_mode.to_string()) // パッケージバージョンを環境変数に設定
        .arg("ipak/scripts/purge.sh"); // 実行するスクリプトのパス

    // パージ処理の実行
    let status = purge_process
        .status()
        .map_err(|e| format!("Failed to execute purge process: {}", e))?;

    // 実行結果の確認
    if status.success() {
        Ok(())
    } else {
        // エラーコードがあればそれも表示
        let code_info = status
            .code()
            .map_or("".to_string(), |c| format!(" (exit code: {})", c));
        Err(format!(
            "Purge process failed with status: {}{}",
            status, code_info
        ))
    }
}
