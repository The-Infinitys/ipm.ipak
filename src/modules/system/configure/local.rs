//! このモジュールは、ローカルシステムにおける`ipak`の初期設定とファイル構造のセットアップを管理します。
//! ユーザーのホームディレクトリ内に必要なディレクトリと設定ファイルを生成します。

use crate::modules::pkg::list;
use crate::utils::files::file_creation;
use serde_yaml;
use std::env;
use std::io::{Error, ErrorKind};
use std::path::Path;

/// ローカルシステムに`ipak`の初期設定を行います。
///
/// ユーザーのホームディレクトリ内に`.ipak`ディレクトリ、`bin`ディレクトリ、
/// `packages`ディレクトリ、`list.yaml`ファイル、`README.md`ファイル、
/// およびキャッシュディレクトリ`.cache/ipak`を作成します。
/// 既存のファイルやディレクトリがある場合はスキップされます。
///
/// # Returns
/// `Ok(())` 成功した場合。
/// `Err(Error)` ファイル操作または環境変数の取得に失敗した場合。
pub fn configure() -> Result<(), Error> {
    let package_list_data = {
        let package_list_data =
            serde_yaml::to_string(&list::PackageListData::default());
        package_list_data.unwrap()
    };
    let configure_list = [
        [".ipak/README.md", include_str!("data/local/README.md")],
        [".ipak/bin/ipak-local", include_str!("data/local/ipak-local")],
        [".ipak/packages/list.yaml", package_list_data.as_str()],
        [".ipak/lock", ""],
        [".ipak/tasks", ""],
    ];
    let home_dir = env::var("HOME").map_err(|e| {
        Error::new(
            ErrorKind::NotFound,
            format!("HOME environment variable not found: {}", e),
        )
    })?;
    let home_dir = Path::new(&home_dir);

    let ipak_bin_dir = home_dir.join(".ipak/bin");
    std::fs::create_dir_all(&ipak_bin_dir).map_err(|e| {
        Error::other(format!(
            "Failed to create .ipak/bin directory: {}",
            e
        ))
    })?;

    for configure_data in configure_list {
        let creation_result = file_creation(
            home_dir.join(configure_data[0]).to_str().unwrap(),
            configure_data[1],
        );
        match creation_result {
            Ok(()) => continue,
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    return Err(e);
                }
            }
        }
    }
    let ipak_cache_dir = home_dir.join(".cache/ipak");
    std::fs::create_dir_all(&ipak_cache_dir).map_err(|e| {
        Error::other(format!(
            "Failed to create .cache/ipak directory: {}",
            e
        ))
    })?;
    Ok(())
}
