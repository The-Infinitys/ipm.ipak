use crate::utils::shell::{self, markdown};
/// Cargo.tomlから取得したパッケージ情報を保持する構造体。
///
/// `CARGO_PKG_NAME`, `CARGO_PKG_VERSION`, `std::env::consts::ARCH` 環境変数から情報を取得します。
struct CargoPackageInfo {
    /// パッケージ名。`CARGO_PKG_NAME` 環境変数から取得。
    name: &'static str,
    /// パッケージのバージョン。`CARGO_PKG_VERSION` 環境変数から取得。
    version: &'static str,
    /// ビルドターゲットのアーキテクチャ。`std::env::consts::ARCH` から取得。
    architecture: &'static str,
}

/// Cargo.tomlからパッケージ情報を取得し、`CargoPackageInfo`構造体として返します。
///
/// コンパイル時に設定される `CARGO_PKG_NAME` および `CARGO_PKG_VERSION`
/// 環境変数を使用します。これらの変数が設定されていない場合は、デフォルト値を使用します。
/// アーキテクチャはコンパイルターゲットの環境定数から取得します。
///
/// # 戻り値
///
/// パッケージ情報を含む `CargoPackageInfo` インスタンス。
fn get_info() -> CargoPackageInfo {
    CargoPackageInfo {
        name: option_env!("CARGO_PKG_NAME").unwrap_or("ipak"),
        version: option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
        architecture: std::env::consts::ARCH,
    }
}

/// 指定されたテキスト内のプレースホルダーをCargoパッケージ情報で置換します。
///
/// プレースホルダーは `{name}`, `{version}`, `{architecture}` の形式です。
///
/// # 引数
///
/// * `text`: プレースホルダーを含む元のテキスト。
///
/// # 戻り値
///
/// プレースホルダーがパッケージ情報で置換された新しい文字列。
fn insert_info(text: &'static str) -> String {
    let cargo_package = get_info();
    let replace_list = vec![
        ["name", cargo_package.name],
        ["version", cargo_package.version],
        ["architecture", cargo_package.architecture],
    ];
    let mut text = text.to_string();
    for replaces in replace_list {
        text = text
            .replace(format!("{{{}}}", replaces[0]).as_str(), replaces[1]);
    }
    text
}

/// マニュアルメッセージを表示します。
/// ページャーを使用します。
pub fn manual() -> Result<(), std::io::Error> {
    let manual_str =
        markdown(insert_info(include_str!("./messages/manual.md")));
    shell::pager(manual_str.to_string());
    Ok(())
}
