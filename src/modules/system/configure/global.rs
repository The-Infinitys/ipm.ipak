//! このモジュールは、グローバルシステムにおける`ipak`の初期設定とファイル構造のセットアップを管理します。
//! システムのルートディレクトリ内に必要なディレクトリと設定ファイルを生成します。

use crate::utils::files::file_creation;
use std::io::{Error, ErrorKind};
use std::path::Path;

/// グローバルシステムに`ipak`の初期設定を行います。
///
/// `/etc/ipak/`ディレクトリ内に`README.md`ファイルを作成します。
/// 既存のファイルやディレクトリがある場合はスキップされます。
///
/// # Returns
/// `Ok(())` 成功した場合。
/// `Err(Error)` ファイル操作に失敗した場合。
pub fn configure() -> Result<(), Error> {
    let configure_list =
        [["/etc/ipak/README.md", include_str!("data/global/README.md")]];
    for configure_data in configure_list {
        let creation_result = file_creation(
            Path::new(configure_data[0]).to_str().unwrap(),
            configure_data[1],
        );
        match creation_result {
            Ok(()) => {
                continue;
            }
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}
