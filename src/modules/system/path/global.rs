//! このモジュールは、グローバルシステムにおける`ipak`関連のパスを管理します。
//! ルートディレクトリ、パッケージリストなどのパスを生成する関数を提供します。

use std::path::PathBuf;

/// システムのルートディレクトリへのパスを返します。
///
/// # Returns
/// システムのルートディレクトリへの`PathBuf`。
fn root_path() -> PathBuf {
    PathBuf::from("/")
}

/// グローバルな`ipak`のルートディレクトリへのパスを返します。
///
/// これは通常、`/etc/ipak/`ディレクトリです。
///
/// # Returns
/// グローバルな`ipak`のルートディレクトリへの`PathBuf`。
fn ipak_path() -> PathBuf {
    root_path().join("etc/ipak/")
}

/// グローバルパッケージリストファイルへのパスを返します。
///
/// # Returns
/// グローバルパッケージリストファイルへの`PathBuf`。
pub fn packageslist_filepath() -> PathBuf {
    packages_dirpath().join("list.yaml")
}

/// グローバルパッケージディレクトリへのパスを返します。
///
/// # Returns
/// グローバルパッケージディレクトリへの`PathBuf`。
pub fn packages_dirpath() -> PathBuf {
    ipak_path().join("packages")
}

/// グローバルロックファイルへのパスを返します。
///
/// # Returns
/// グローバルロックファイルへの`PathBuf`。
pub fn lock_filepath() -> PathBuf {
    ipak_path().join("lock")
}

/// グローバルタスクファイルへのパスを返します。
///
/// # Returns
/// グローバルタスクファイルへの`PathBuf`。
pub fn tasks_filepath() -> PathBuf {
    ipak_path().join("tasks")
}
