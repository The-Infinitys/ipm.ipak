//! このモジュールは、`ipak`のシステム設定（ローカルおよびグローバル）を管理します。
//! ユーザーの権限に基づいて適切な設定関数を呼び出します。

mod global;
mod local;
use crate::utils::shell::is_superuser;

/// `ipak`のシステム設定を行います。
///
/// `arg`が`Some(true)`の場合、ローカル設定を行います。
/// `arg`が`Some(false)`の場合、グローバル設定を行います。
/// `arg`が`None`の場合、現在のユーザーがスーパーユーザーであればグローバル設定を、
/// そうでなければローカル設定を行います。
///
/// # Arguments
/// * `arg` - 設定の種類を指定する`Option<bool>`。
///
/// # Returns
/// `Ok(())` 成功した場合。
/// `Err(std::io::Error)` 設定中にエラーが発生した場合。
pub fn configure(arg: Option<bool>) -> Result<(), std::io::Error> {
    match arg {
        Some(is_local) => {
            if is_local {
                local::configure()
            } else {
                global::configure()
            }
        }
        None => {
            if is_superuser() {
                global::configure()
            } else {
                local::configure()
            }
        }
    }
}
