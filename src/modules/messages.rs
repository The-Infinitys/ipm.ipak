//! このモジュールは、アプリケーションのメッセージ（ウェルカムメッセージ、バージョン情報、ヘルプメッセージなど）の表示を扱います。
//! Cargo.tomlからパッケージ情報を取得し、メッセージテンプレート内のプレースホルダーを置換する機能を提供します。
use crate::utils::shell;
use cmd_arg::cmd_arg::{Option, cmd_str}; // `Option`構造体が外部モジュールにあることを示しています
use termimad::crossterm::style::{Attribute::*, Color::*};
use termimad::*; // `termimad`は、ターミナルでのテキスト表示を扱うライブラリです。
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

/// ウェルカムメッセージを表示します。
///
/// `messages/welcome.txt` ファイルの内容を読み込み、パッケージ情報でプレースホルダーを置換して標準出力に表示します。
pub fn welcome() {
    let welcome_str = insert_info(include_str!("./messages/welcome.txt"));
    println!("{}", welcome_str);
}

/// アプリケーションのバージョン情報を表示します。
///
/// `[name] [version] ([architecture])` の形式で標準出力に表示します。
pub fn version() -> Result<(), std::io::Error> {
    let cargo_package = get_info();
    println!(
        "{} {} ({})",
        cargo_package.name,
        cargo_package.version,
        cargo_package.architecture
    );
    Ok(())
}

/// ヘルプメッセージを表示します。
///
/// 引数に基づいて表示するヘルプメッセージの種類を決定します。
/// 引数がない場合や不明な場合は一般的なヘルプを表示します。
///
/// # 引数
///
/// * `args`: ヘルプメッセージの種類を決定するために使用される引数のベクタ。
///   現在の実装では、最初の引数のみが `install` かどうかの判定に使用されます。
pub fn help(args: Vec<&Option>) -> Result<(), std::io::Error> {
    let help_type: HelpType = if args.is_empty() {
        HelpType::Main
    } else {
        match args[0].opt_str.as_str() {
            "help" => HelpType::Help,
            "manual" | "man" => HelpType::Manual,
            "version" => HelpType::Version,
            "project" => HelpType::Project,
            "system" => HelpType::System,
            "package" => HelpType::Package,
            _ => HelpType::Help,
        }
    };
    show_help(help_type);
    Ok(())
}

/// 表示するヘルプメッセージの種類を表す列挙型。
enum HelpType {
    /// 一般的なヘルプメッセージ。
    Main,
    Help,
    Manual,
    Version,
    Project,
    Package,
    System,
}

/// 指定されたヘルプメッセージの種類に対応するテキストを取得します。
///
/// 対応する `.txt` ファイルの内容を読み込み、パッケージ情報でプレースホルダーを置換します。
///
/// # 引数
///
/// * `help_type`: 取得するヘルプメッセージの種類。
///
/// # 戻り値
///
/// プレースホルダーが置換されたヘルプメッセージの文字列。
fn get_help_msg(help_type: HelpType) -> String {
    insert_info(match help_type {
        HelpType::Main => include_str!("./messages/help/main.md"),
        HelpType::Help => include_str!("./messages/help/help.md"),
        HelpType::Manual => include_str!("./messages/help/manual.md"),
        HelpType::Version => include_str!("./messages/help/version.md"),
        HelpType::Project => include_str!("./messages/help/project.md"),
        HelpType::Package => include_str!("./messages/help/package.md"),
        HelpType::System => include_str!("./messages/help/system.md"),
    })
}
fn markdown(md_text: String) -> String {
    let mut skin = MadSkin::default();
    // let's decide bold is in light gray
    skin.bold.set_fg(gray(20));
    // let's make strikeout not striked out but red, with no specific background, and bold
    skin.strikeout = CompoundStyle::new(Some(Red), None, Bold.into());
    format!("{}", skin.term_text(&md_text))
}
/// 指定されたヘルプメッセージの種類に対応するテキストを標準出力に表示します。
///
/// # 引数
///
/// * `help_type`: 表示するヘルプメッセージの種類。
fn show_help(help_type: HelpType) {
    let help_msg = markdown(get_help_msg(help_type));
    println!("{}", help_msg);
}

/// 不明なヘルプタイプが指定された場合にエラーメッセージを表示します。
pub fn unknown() -> Result<(), std::io::Error> {
    eprintln!("unknown command:\n  {}", cmd_str());
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "No HelpType provided",
    ))
}
/// マニュアルメッセージを表示します。
/// ページャー
pub fn manual() -> Result<(), std::io::Error> {
    let manual_str =
        markdown(insert_info(include_str!("./messages/manual.md")));
    shell::pager(manual_str.to_string());
    Ok(())
}
