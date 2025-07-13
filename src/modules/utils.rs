//! このモジュールは、様々なユーティリティコマンドのディスパッチと実行を処理します。
//! 主にアーカイブ関連の操作を扱います。

use crate::utils::archive::{create_archive, extract_archive};
use crate::utils::args::{ArchiveCommands, UtilsCommands};
use crate::utils::error::IpakError;

/// ユーティリティコマンドを処理します。
///
/// `UtilsCommands`列挙型に基づいて、適切なユーティリティ関数にディスパッチします。
///
/// # Arguments
/// * `args` - 処理するユーティリティコマンド。
///
/// # Returns
/// `Ok(())` 成功した場合。
/// `Err(IpakError)` エラーが発生した場合。
pub fn utils(args: UtilsCommands) -> Result<(), IpakError> {
    match args {
        UtilsCommands::Archive(args) => archive(args)?,
    }
    Ok(())
}

/// アーカイブ関連のコマンドを処理します。
///
/// `ArchiveCommands`列挙型に基づいて、アーカイブの作成または展開を実行します。
///
/// # Arguments
/// * `args` - 処理するアーカイブコマンド。
///
/// # Returns
/// `Ok(())` 成功した場合。
/// `Err(IpakError)` エラーが発生した場合。
fn archive(args: ArchiveCommands) -> Result<(), IpakError> {
    match args {
        ArchiveCommands::Create { from, to, archive_type } => {
            create_archive(&from, &to, archive_type)
                .map_err(IpakError::from)?
        }
        ArchiveCommands::Extract { from, to } => {
            extract_archive(&from, &to).map_err(IpakError::from)?
        }
    }
    Ok(())
}
