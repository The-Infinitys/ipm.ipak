use crate::{modules::pkg::PackageData, utils::files::file_creation};
use crate::utils::shell;
use std::{
    io::{self, Error, ErrorKind},
    process::Command,
};

/// プロジェクトのセットアップに必要なファイルパスとコンテンツを保持する構造体。
///
/// この構造体は、テンプレートファイルパスとその内容を関連付けます。
struct SetUpItem {
    path: String,
    content: String,
}

/// 指定されたファイルリストに基づいてファイルを生成します。
///
/// 各ファイルは、そのパスとコンテンツに従って作成されます。
/// ファイル作成中にエラーが発生した場合、具体的なエラーメッセージと共に
/// `std::io::Error` が返されます。
///
/// # 引数
///
/// * `setup_list` - 生成するファイルのパスとコンテンツのリスト。
///
/// # 戻り値
///
/// ファイル生成がすべて成功した場合は `Ok(())`、一つでも失敗した場合は `std::io::Error` を返します。
fn setup_files(setup_list: Vec<SetUpItem>) -> Result<(), io::Error> {
    for item in setup_list {
        // file_creation の結果を直接伝播させ、エラー発生時に詳細な情報を付与する
        file_creation(&item.path, &item.content).map_err(|e| {
            Error::new(
                e.kind(),
                format!("Failed to create file '{}': {}", item.path, e),
            )
        })?;
    }
    Ok(())
}

/// デフォルトのプロジェクトテンプレートを設定します。
///
/// このテンプレートには、基本的なシェルスクリプト (`src/main.sh`) と、
/// `ipak/scripts/` ディレクトリ内にビルド、インストール、削除、パージの各スクリプトが含まれます。
/// これらは新しいプロジェクトの初期構造を提供します。
///
/// # 戻り値
///
/// テンプレートの設定が成功した場合は `Ok(())`、ファイル作成に失敗した場合は `std::io::Error` を返します。
pub fn default(pkg_data:PackageData) -> Result<PackageData, io::Error> {
    let setup_list = vec![
        SetUpItem {
            path: "ipak/scripts/build.sh".to_string(),
            content: include_str!(
                "templates/default/ipak/scripts/build.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/install.sh".to_string(),
            content: include_str!(
                "templates/default/ipak/scripts/install.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/remove.sh".to_string(),
            content: include_str!(
                "templates/default/ipak/scripts/remove.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/purge.sh".to_string(),
            content: include_str!(
                "templates/default/ipak/scripts/purge.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/README.md".to_string(),
            content: include_str!("templates/script-README.md")
                .to_string(),
        },
    ];
    setup_files(setup_list)
}

/// Rust プロジェクトテンプレートを設定します。
///
/// この関数は、最初にシステムに `cargo` コマンド（Rustのパッケージマネージャー）が
/// インストールされているかを確認します。`cargo` が利用可能な場合、`cargo init` を実行して
/// 標準的なRustプロジェクト構造を初期化し、その後、ipak固有のビルド、インストール、
/// 削除、パージスクリプトを `ipak/scripts/` ディレクトリ内に配置します。
///
/// # 戻り値
///
/// テンプレートの設定が成功した場合は `Ok(())`、`cargo` が見つからない場合や
/// コマンドの実行に失敗した場合は `std::io::Error` を返します。
pub fn rust(pkg_data:PackageData) -> Result<PackageData, io::Error> {
    // 'cargo' コマンドの利用可能性をチェック
    if !shell::is_cmd_available("cargo") {
        let rustup_url = "https://www.rust-lang.org/tools/install";
        eprintln!("Error: 'cargo' command not found.");
        eprintln!(
            "To create a Rust project, you need to install Cargo (Rust's package manager)."
        );
        eprintln!(
            "Please visit {} for installation instructions.",
            rustup_url
        );
        return Err(Error::new(
            ErrorKind::NotFound,
            "Cargo command not found. Please install Rust and Cargo.",
        ));
    }

    // 'cargo init' を実行してRustプロジェクトを初期化
    let status =
        Command::new("cargo").arg("init").status().map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to execute 'cargo init': {}", e),
            )
        })?;

    if !status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "'cargo init' command failed with exit status: {}",
                status
            ),
        ));
    }

    // ipak スクリプトをRustプロジェクトに追加
    let setup_list = vec![
        SetUpItem {
            path: "ipak/scripts/build.sh".to_string(),
            content: include_str!("templates/rust/ipak/scripts/build.sh")
                .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/install.sh".to_string(),
            content: include_str!(
                "templates/rust/ipak/scripts/install.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/remove.sh".to_string(),
            content: include_str!("templates/rust/ipak/scripts/remove.sh")
                .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/purge.sh".to_string(),
            content: include_str!("templates/rust/ipak/scripts/purge.sh")
                .to_string(),
        },
        SetUpItem {
            path: "ipak/project-ignore.yaml".to_string(),
            content: include_str!(
                "templates/rust/ipak/project-ignore.yaml"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/README.md".to_string(),
            content: include_str!("templates/script-README.md")
                .to_string(),
        },
    ];
    setup_files(setup_list)
}

/// Python プロジェクトテンプレートを設定します。
///
/// この関数は、最初にシステムに `python3` コマンドがインストールされているかを確認します。
/// `python3` が利用可能な場合、`python3 -m venv venv` を実行して仮想環境を初期化し、
/// その後、基本的なPythonプロジェクトファイル (`src/main.py`, `requirements.txt`, `.gitignore`) と、
/// ipak固有のビルド、インストール、削除、パージスクリプトを `ipak/scripts/` ディレクトリ内に配置します。
///
/// # 戻り値
///
/// テンプレートの設定が成功した場合は `Ok(())`、`python3` が見つからない場合や
/// コマンドの実行またはファイル作成に失敗した場合は `std::io::Error` を返します。
pub fn python(pkg_data:PackageData) -> Result<PackageData, io::Error> {
    // 'python3' コマンドの利用可能性をチェック
    if !shell::is_cmd_available("python3") {
        let python_url = "https://www.python.org/downloads/";
        eprintln!("Error: 'python3' command not found.");
        eprintln!(
            "To create a Python project, you need to install Python 3."
        );
        eprintln!(
            "Please visit {} for installation instructions.",
            python_url
        );
        return Err(Error::new(
            ErrorKind::NotFound,
            "python3 command not found. Please install Python 3.",
        ));
    }

    // 'python3 -m venv venv' を実行して仮想環境を初期化
    // これは 'cargo init' がプロジェクト環境を作成するのに似ています。
    let venv_status = Command::new("python3")
        .args(["-m", "venv", "venv"]) // 'venv' という名前のフォルダを作成します
        .status()
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to execute 'python3 -m venv venv': {}", e),
            )
        })?;

    if !venv_status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "'python3 -m venv venv' command failed with exit status: {}",
                venv_status
            ),
        ));
    }
    eprintln!(
        "Virtual environment 'venv' created successfully in the current directory."
    );

    // ipak スクリプトと基本的なPythonファイルをプロジェクトに追加
    let setup_list = vec![
        // ipak スクリプト (Pythonプロジェクト向け)
        SetUpItem {
            path: "ipak/scripts/build.sh".to_string(),
            content: include_str!(
                "templates/python/ipak/scripts/build.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/install.sh".to_string(),
            content: include_str!(
                "templates/python/ipak/scripts/install.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/remove.sh".to_string(),
            content: include_str!(
                "templates/python/ipak/scripts/remove.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/purge.sh".to_string(),
            content: include_str!(
                "templates/python/ipak/scripts/purge.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/README.md".to_string(),
            content: include_str!("templates/script-README.md")
                .to_string(), // 共通のREADMEを使用
        },
        SetUpItem {
            path: "src/main.py".to_string(),
            content: include_str!("templates/python/src/main.py")
                .to_string(),
        },
        SetUpItem {
            path: "requirements.txt".to_string(),
            content: include_str!("templates/python/requirements.txt")
                .to_string(),
        },
    ];
    setup_files(setup_list)
}

pub fn dotnet(pkg_data:PackageData) -> Result<PackageData, io::Error> {
    // 'dotnet' コマンドの利用可能性をチェック
    if !shell::is_cmd_available("dotnet") {
        let dotnet_url = "https://dotnet.microsoft.com/download";
        eprintln!("Error: 'dotnet' command not found.");
        eprintln!("To create a .NET project, you need to install .NET");
        eprintln!("For more information, please visit {}.", dotnet_url);
        return Err(Error::new(
            ErrorKind::NotFound,
            "dotnet command not found. Please install .NET.",
        ));
    }
    

    // 'dotnet new' を実行してDotnetプロジェクトを初期化
    let status =
        Command::new("dotnet").arg("new").arg("console").arg("--output=./").status().map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to execute 'dotnet new': {}", e),
            )
        })?;

    if !status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "'dotnet new' command failed with exit status: {}",
                status
            ),
        ));
    }

    // ipak スクリプトをdotnetプロジェクトに追加
    let setup_list = vec![
        SetUpItem {
            path: "ipak/scripts/build.sh".to_string(),
            content: include_str!("templates/dotnet/ipak/scripts/build.sh")
                .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/install.sh".to_string(),
            content: include_str!(
                "templates/dotnet/ipak/scripts/install.sh"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/remove.sh".to_string(),
            content: include_str!("templates/dotnet/ipak/scripts/remove.sh")
                .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/purge.sh".to_string(),
            content: include_str!("templates/dotnet/ipak/scripts/purge.sh")
                .to_string(),
        },
        SetUpItem {
            path: "ipak/project-ignore.yaml".to_string(),
            content: include_str!(
                "templates/dotnet/ipak/project-ignore.yaml"
            )
            .to_string(),
        },
        SetUpItem {
            path: "ipak/scripts/README.md".to_string(),
            content: include_str!("templates/script-README.md")
                .to_string(),
        },
    ];
    setup_files(setup_list)?;
    Ok(())
}
