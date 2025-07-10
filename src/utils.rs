//! このモジュールは、アプリケーション全体で利用される様々なユーティリティ機能を提供します。
//! ファイル操作、シェルコマンド実行、デバッグ出力、エラーハンドリング、コマンドライン引数解析、アーカイブ処理、カラー出力など、多岐にわたる補助的な機能が含まれています。

/// アーカイブユーリティ
/// 自動でアーカイブの種類を識別し解凍したり、アーカイブを作成したりできます。
pub mod archive;
/// 引数を管理しています。
pub mod args;
/// カラー出力ユーリティ
pub mod color;
/// デバッグユーリティ
pub mod debug;
/// エラーの基盤となります。
pub mod error;
/// ファイル操作をします。
pub mod files;
/// シェル・ユーリティ
pub mod shell;
/// バージョニングおよびバージョン範囲の管理を処理します。
pub mod version;


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
