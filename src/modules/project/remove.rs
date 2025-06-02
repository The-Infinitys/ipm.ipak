use super::ExecMode;
use super::ExecShell;
use super::metadata; // metadata モジュール全体をインポート
use crate::dprintln;
use crate::utils::color::colorize::*;
use std::fmt::{self, Display};
/// プロジェクト削除時のオプションを定義する構造体です。
#[derive(Default)]
pub struct RemoveOptions {
    /// 削除スクリプトを実行するシェルを指定します。
    pub remove_shell: ExecShell,
    pub remove_mode: ExecMode,
}

impl Display for RemoveOptions {
    /// `RemoveOptions` をフォーマットして表示します。
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

/// プロジェクトを削除する関数です。
///
/// この関数は、`ipak` プロジェクトのメタデータを読み込み、
/// 指定されたシェルで削除スクリプト (`ipak/scripts/remove.sh`) を実行します。
/// 削除プロセスが成功した場合は `Ok(())` を、失敗した場合は `Err(String)` を返します。
///
/// # 引数
///
/// * `opts`: 削除オプション (`RemoveOptions`)。
///
/// # 戻り値
///
/// 削除プロセスが成功した場合は `Ok(())`、失敗した場合はエラーメッセージを含む `Err(String)`。
pub fn remove(opts: RemoveOptions) -> Result<(), String> {
    dprintln!("{}", &opts); // デバッグログ出力

    // Ipak ディレクトリのパスを取得します。
    // エラーの場合は、より詳細なエラーメッセージを返します。
    let target_dir = metadata::get_dir().map_err(|_| {
        "Error: Couldn't find Ipak Directory. Make sure you are in a project directory or Ipak is installed."
            .to_string()
    })?;

    // プロジェクトのメタデータを取得します。
    // `metadata().unwrap()` はパニックの可能性があるため、`?` を使用してエラーを伝播させます。
    let project_metadata = metadata::metadata() // `metadata::metadata` を明示的に呼び出す
        .map_err(|e| {
            format!("Error: Failed to retrieve project metadata: {:?}", e)
        })?;

    // 削除スクリプトを実行するためのコマンドを設定します。
    let mut remove_process = opts.remove_shell.generate();

    // コマンドの作業ディレクトリ、環境変数、および実行するスクリプトを設定します。
    remove_process
        .current_dir(&target_dir)
        .env("IPAK_PROJECT_NAME", &project_metadata.about.package.name)
        .env(
            "IPAK_PROJECT_VERSION",
            project_metadata.about.package.version.to_string(),
        )
        .env("IPAK_REMOVE_MODE", opts.remove_mode.to_string())
        .arg("ipak/scripts/remove.sh");

    // 削除プロセスを実行し、結果をハンドリングします。
    let status = remove_process
        .status()
        .map_err(|e| format!("Failed to execute remove process: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        // エラーコードがあればそれを表示し、なければ一般的な失敗メッセージを表示します。
        Err(format!(
            "Remove process failed with status: {}",
            status.code().unwrap_or(-1) // ExitStatus からコードを取得
        ))
    }
}
