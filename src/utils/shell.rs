pub mod question;
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use termimad::crossterm::style::{Attribute::*, Color::*};
use termimad::*;

/// 指定されたコマンドがシステムで利用可能かどうかを確認します。
///
/// # Arguments
/// * `cmd` - チェックするコマンド名
///
/// # Returns
/// `true`: コマンドが利用可能な場合
/// `false`: コマンドが利用できない場合
pub fn is_cmd_available(cmd: &str) -> bool {
    let path_env = env::var("PATH");
    match path_env {
        Ok(path_env) => {
            for check_path in path_env.split(":") {
                if Path::new(check_path).join(cmd).is_file() {
                    return true;
                }
            }
        }
        Err(e) => {
            log::error!("PATH環境変数の読み取りに失敗しました: {}", e);
        }
    }
    false
}

/// 現在のユーザー名を取得します。
///
/// # Returns
/// 現在のユーザー名を表す文字列
pub fn username() -> String {
    let output = Command::new("whoami")
        .output()
        .expect("whoamiコマンドの実行に失敗しました");

    let username = String::from_utf8(output.stdout)
        .expect("whoamiコマンドの出力が不正なUTF-8です");

    if cfg!(target_os = "windows") {
        // Windowsの場合、出力は通常 'DOMAIN\username' 形式
        username.split('\\').next_back().unwrap_or("").trim().to_string()
    } else {
        // Linux/macOSの場合、出力は直接ユーザー名
        username.trim().to_string()
    }
}

/// ホスト名を取得します。
///
/// # Returns
/// ホスト名を表す文字列
pub fn hostname() -> String {
    let output = Command::new("hostname")
        .output()
        .expect("hostnameコマンドの実行に失敗しました");
    String::from_utf8(output.stdout)
        .expect("hostnameコマンドの出力が不正なUTF-8です")
        .trim()
        .to_string()
}

/// 現在のシェルタイプを取得します。
///
/// # Returns
/// シェルタイプを表す文字列 (例: "bash", "zsh", "fish")
pub fn shell_type() -> String {
    env::var("SHELL")
        .unwrap_or_else(|_| "unknown".to_string())
        .split('/')
        .next_back()
        .unwrap_or("unknown")
        .to_string()
}

/// 現在のユーザーがスーパーユーザー (root) かどうかを判定します。
/// Windowsでは常に`false`を返します。
///
/// # Returns
/// `true`: スーパーユーザーの場合
/// `false`: スーパーユーザーではない場合
pub fn is_superuser() -> bool {
    if cfg!(target_os = "windows") {
        return false;
    }
    let output = Command::new("id")
        .output()
        .expect("idコマンドの実行に失敗しました");
    let id = String::from_utf8(output.stdout)
        .expect("idコマンドの出力が不正なUTF-8です");
    id.contains("uid=0(root)")
}

/// 指定された文字列をページャーで表示します。
/// 環境変数`PAGER`が設定されていればそのコマンドを使用し、
/// なければ`less`を使用します。
/// ページャーの起動に失敗した場合は、直接標準出力に表示します。
///
/// # Arguments
/// * `target_string` - 表示する文字列
pub fn pager(target_string: String) {
    let pager_command_str =
        std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());

    let pager_name = Path::new(&pager_command_str)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&pager_command_str)
        .to_lowercase();

    let mut command = Command::new(&pager_command_str);

    // lessの場合、色付き出力などのオプションを追加
    if pager_name == "less" {
        command.arg("-R").arg("-F").arg("-X").arg("-");
    }

    let mut child_result = command.stdin(Stdio::piped()).spawn();

    // 特定の引数でページャーの起動に失敗した場合、引数なしで再試行
    if let Err(ref e) = child_result {
        log::error!(
            "Warning: Pager '{}' failed to start with specific arguments ({}). Retrying without arguments.",
            pager_command_str, e
        );
        command = Command::new(&pager_command_str);
        child_result = command.stdin(Stdio::piped()).spawn();
    }

    let mut child = match child_result {
        Ok(child) => child,
        Err(e) => {
            log::error!(
                "Error: Pager '{}' failed to start ({}). Printing directly to stdout.",
                pager_command_str, e
            );
            io::stdout()
                .write_all(target_string.as_bytes())
                .expect("Failed to write to stdout");
            return;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(target_string.as_bytes()) {
            log::error!(
                "Error: Failed to write to pager '{}' stdin ({}). Printing directly to stdout.",
                pager_command_str, e
            );
            io::stdout()
                .write_all(target_string.as_bytes())
                .expect("Failed to write to stdout");
            return;
        }
    } else {
        log::error!(
            "Error: Failed to open pager '{}' stdin. Printing directly to stdout.",
            pager_command_str
        );
        io::stdout()
            .write_all(target_string.as_bytes())
            .expect("Failed to write to stdout");
        return;
    }

    // ページャープロセスの終了を待機
    let output = child
        .wait_with_output()
        .expect("failed to wait for pager process");

    if !output.status.success() && !output.stderr.is_empty() {
        log::error!(
            "Pager '{}' exited with error: {}",
            pager_command_str,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Markdownテキストをターミナル表示用に整形します。
///
/// # Arguments
/// * `md_text` - 整形するMarkdown形式の文字列
///
/// # Returns
/// 整形された文字列
pub fn markdown(md_text: &str) -> String {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(gray(20));
    skin.strikeout = CompoundStyle::new(Some(Red), None, Bold.into());
    format!("{}", skin.term_text(md_text))
}
