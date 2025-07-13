//! このモジュールは、ローカルシステムにおける`ipak`関連のパスを管理します。
//! ホームディレクトリ、パッケージリスト、キャッシュなどのパスを生成する関数を提供します。

use crate::utils::shell;
use std::env;
use std::path::PathBuf;

/// ユーザーのホームディレクトリへのパスを返します。
///
/// `HOME`環境変数が設定されていない場合は、ユーザー名に基づいてパスを推測します。
///
/// # Returns
/// ユーザーのホームディレクトリへの`PathBuf`。
fn home_path() -> PathBuf {
    let home_path_str = env::var("HOME").unwrap_or_else(|_| {
        log::error!("IpakError: HOME environment variable not set. Attempting to use username.");
        let username = shell::username();
        format!("/home/{}", username)
    });
    PathBuf::from(home_path_str)
}

/// `ipak`のルートディレクトリへのパスを返します。
///
/// これは通常、ホームディレクトリ内の`.ipak`ディレクトリです。
///
/// # Returns
/// `ipak`のルートディレクトリへの`PathBuf`。
fn ipak_path() -> PathBuf {
    home_path().join(".ipak")
}

/// ローカルパッケージリストファイルへのパスを返します。
///
/// # Returns
/// ローカルパッケージリストファイルへの`PathBuf`。
pub fn packageslist_filepath() -> PathBuf {
    packages_dirpath().join("list.yaml")
}

/// ローカルパッケージディレクトリへのパスを返します。
///
/// # Returns
/// ローカルパッケージディレクトリへの`PathBuf`。
pub fn packages_dirpath() -> PathBuf {
    ipak_path().join("packages")
}

/// `ipak`のキャッシュディレクトリへのパスを返します。
///
/// # Returns
/// `ipak`のキャッシュディレクトリへの`PathBuf`。
pub fn cache_path() -> PathBuf {
    home_path().join(".cache/ipak/")
}

/// `ipak`のロックファイルへのパスを返します。
///
/// # Returns
/// `ipak`のロックファイルへの`PathBuf`。
pub fn lock_filepath() -> PathBuf {
    ipak_path().join("lock")
}

/// `ipak`のタスクファイルへのパスを返します。
///
/// # Returns
/// `ipak`のタスクファイルへの`PathBuf`。
pub fn tasks_filepath() -> PathBuf {
    ipak_path().join("tasks")
}
