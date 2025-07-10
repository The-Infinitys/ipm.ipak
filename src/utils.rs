//! このモジュールは、アプリケーション全体で利用される様々なユーティリティ機能を提供します。
//! ファイル操作、シェルコマンド実行、デバッグ出力、エラーハンドリング、コマンドライン引数解析、アーカイブ処理、カラー出力など、多岐にわたる補助的な機能が含まれています。

pub mod archive;
pub mod args;
pub mod color;
pub mod debug;
pub mod error;
pub mod files;
pub mod shell;

/// 現在のユーザー名とホスト名に基づいてローカルのメールアドレスを生成します。
///
/// # Returns
/// 生成されたメールアドレスの文字列 (例: "username@hostname.local")
pub fn generate_email_address() -> String {
    let username = shell::username();
    let hostname = shell::hostname();
    let domain = "local";
    format!("{}@{}.{}", username, hostname, domain)
}
