//! このモジュールは、システム関連のコマンドを処理します。
//! 主に`ipak`の設定（ローカルおよびグローバル）を管理します。

mod configure;
pub mod path;
use crate::utils::{args::SystemCommands, error::Error};

/// システム関連のコマンドを処理します。
///
/// `SystemCommands`列挙型に基づいて、適切なシステム関数にディスパッチします。
/// 現在は設定コマンドのみをサポートしています。
///
/// # Arguments
/// * `args` - 処理するシステムコマンド。
///
/// # Returns
/// `Ok(())` 成功した場合。
/// `Err(Error)` エラーが発生した場合。
pub fn system(args: SystemCommands) -> Result<(), Error> {
    match args {
        SystemCommands::Configure { local, global } => {
            configure::configure({
                if local && !global {
                    Some(true)
                } else if global && !local {
                    Some(false)
                } else {
                    None
                }
            })
            .map_err(|e| Error::from(e))?
        }
    }
    Ok(())
}
