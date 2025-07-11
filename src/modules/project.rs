//! このモジュールは、`ipak`プロジェクトの管理に関連する機能を提供します。
//! プロジェクトの作成、ビルド、インストール、削除、パージ、実行、メタデータ表示など、
//! プロジェクトライフサイクル全体をカバーするコマンドを定義します。

use crate::modules::project::build::BuildMode;
use crate::modules::project::package::PackageTarget;
use crate::utils::error::Error;
use crate::utils::shell::is_superuser;
use crate::utils::{
    args::ProjectCommands,
    generate_email_address,
    shell::{self, username},
};
use build::BuildOptions;
use install::InstallOptions;
use purge::PurgeOptions;
use remove::RemoveOptions;
use std::{env, fs, str::FromStr};
pub mod build;
pub mod create;
mod init;
pub mod install;
pub mod metadata;
pub mod package;
pub mod purge;
pub mod remove;
pub mod run;
use super::pkg::AuthorAboutData;
use clap;
use create::ProjectParams;
pub use create::ProjectTemplateType;
use std::fmt::{self, Display};
use std::process::Command;

/// 実行モード（ローカルまたはグローバル）を定義する列挙型です。
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ExecMode {
    /// ローカルモードでの実行。
    Local,
    /// グローバルモードでの実行。
    Global,
}

impl From<(bool, bool)> for ExecMode {
    /// `(local: bool, global: bool)`タプルから`ExecMode`を変換します。
    ///
    /// `local`が`true`で`global`が`false`の場合、`Local`を返します。
    /// `global`が`true`で`local`が`false`の場合、`Global`を返します。
    /// それ以外の場合（両方`true`、両方`false`など）は、現在のユーザーがスーパーユーザーであれば`Global`、
    /// そうでなければ`Local`を返します。
    fn from(value: (bool, bool)) -> Self {
        let (local, global) = value;
        if local && !global {
            ExecMode::Local
        } else if global && !local {
            ExecMode::Global
        } else {
            ExecMode::from(None)
        }
    }
}

impl From<Option<bool>> for ExecMode {
    /// `Option<bool>`から`ExecMode`を変換します。
    ///
    /// `Some(true)`の場合、`Local`を返します。
    /// `Some(false)`の場合、`Global`を返します。
    /// `None`の場合、現在のユーザーがスーパーユーザーであれば`Global`、
    /// そうでなければ`Local`を返します。
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(is_local) => {
                if is_local {
                    ExecMode::Local
                } else {
                    ExecMode::Global
                }
            }
            None => {
                if is_superuser() {
                    ExecMode::Global
                } else {
                    ExecMode::Local
                }
            }
        }
    }
}

impl Display for ExecMode {
    /// `ExecMode`を整形して表示します。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecMode::Local => write!(f, "local"),
            ExecMode::Global => {
                write!(f, "global")
            }
        }
    }
}

impl Default for ExecMode {
    /// デフォルトの`ExecMode`を返します。
    ///
    /// 現在のユーザーがスーパーユーザーであれば`Global`、そうでなければ`Local`を返します。
    fn default() -> Self {
        if shell::is_superuser() { Self::Global } else { Self::Local }
    }
}

/// 実行に使用するシェルを定義する列挙型です。
#[derive(Default, clap::ValueEnum, Clone, Copy, Debug)]
pub enum ExecShell {
    /// 制限付きBashシェル。
    RBash,
    #[default]
    /// Bashシェル。
    Bash,
    /// Zshシェル。
    Zsh,
    /// Cshシェル。
    Csh,
}

impl FromStr for ExecShell {
    type Err = String;
    /// 文字列から`ExecShell`をパースします。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "bash" => Ok(Self::Bash),
            "zsh" => Ok(Self::Zsh),
            "csh" => Ok(Self::Csh),
            "rbash" => Ok(Self::RBash),
            _ => Err(format!("Unavailable Shell: {}", s)),
        }
    }
}

impl ExecShell {
    /// 実行可能な`Command`オブジェクトを生成します。
    ///
    /// # Returns
    /// 実行可能な`Command`オブジェクト。
    pub fn generate(&self) -> Command {
        match self {
            Self::RBash => {
                let mut cmd = Command::new("bash");
                cmd.arg("-r");
                cmd
            }
            Self::Bash => Command::new("bash"),
            Self::Zsh => Command::new("zsh"),
            Self::Csh => Command::new("csh"),
        }
    }
}

impl Display for ExecShell {
    /// `ExecShell`を整形して表示します。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecShell::RBash => {
                write!(f, "restricted bash")
            }
            ExecShell::Bash => write!(f, "bash"),
            ExecShell::Zsh => write!(f, "zsh"),
            ExecShell::Csh => write!(f, "csh"),
        }
    }
}

/// プロジェクト関連のコマンドを処理します。
///
/// `ProjectCommands`列挙型に基づいて、適切なプロジェクト管理関数にディスパッチします。
///
/// # Arguments
/// * `args` - 処理するプロジェクトコマンド。
///
/// # Returns
/// `Ok(())` 成功した場合。
/// `Err(Error)` エラーが発生した場合。
pub fn project(args: ProjectCommands) -> Result<(), Error> {
    match args {
        ProjectCommands::Create {
            project_name,
            template,
            author_name,
            author_email,
        } => project_create(
            project_name,
            template,
            author_name,
            author_email,
        ),
        ProjectCommands::Metadata => project_metadata(),
        ProjectCommands::Build { release, shell } => {
            project_build(release, shell)
        }
        ProjectCommands::Install { global, shell } => {
            project_install(global, shell)
        }
        ProjectCommands::Remove { local, global, shell } => {
            project_remove((local, global).into(), shell)
        }
        ProjectCommands::Purge { local, global, shell } => {
            project_purge((local, global).into(), shell)
        }
        ProjectCommands::Package { target } => project_package(target),
        ProjectCommands::Init => project_init(),
        ProjectCommands::Run { shell, command, args } => {
            project_run(shell, command, args)
        }
    }
}

// 以下の関数を `pub` に変更
pub fn project_run(
    shell: Option<ExecShell>,
    command: String,
    args: Vec<String>,
) -> Result<(), Error> {
    run::run(shell, &command, args).map_err(Error::from)
}

/// プロジェクトを初期化します。
///
/// `init`モジュールの`init`関数を呼び出します。
///
/// # Returns
/// `Ok(())` 初期化が正常に完了した場合。
/// `Err(Error)` 初期化中にエラーが発生した場合。
pub fn project_init() -> Result<(), Error> {
    init::init().map_err(Error::from)
}

/// プロジェクトをパッケージ化します。
///
/// `package`モジュールの`package`関数を呼び出します。
///
/// # Arguments
/// * `target` - パッケージ化のターゲット（オプション）。
///
/// # Returns
/// `Ok(())` パッケージ化が正常に完了した場合。
/// `Err(Error)` パッケージ化中にエラーが発生した場合。
pub fn project_package(target: Option<PackageTarget>) -> Result<(), Error> {
    let package_options =
        package::PackageOptions { target: target.unwrap_or_default() };

    package::package(package_options).map_err(Error::from)
}

/// プロジェクトをビルドします。
///
/// `build`モジュールの`build`関数を呼び出します。
///
/// # Arguments
/// * `release` - リリースモードでビルドするかどうか。
/// * `shell` - ビルドに使用するシェル（オプション）。
///
/// # Returns
/// `Ok(())` ビルドが正常に完了した場合。
/// `Err(Error)` ビルド中にエラーが発生した場合。
pub fn project_build(
    release: bool,
    shell: Option<ExecShell>,
) -> Result<(), Error> {
    let build_options = BuildOptions {
        build_mode: if release {
            BuildMode::Release
        } else {
            BuildMode::Debug
        },
        build_shell: shell.unwrap_or_default(),
    };
    build::build(build_options).map_err(Error::from)
}

/// プロジェクトをインストールします。
///
/// `install`モジュールの`install`関数を呼び出します。
///
/// # Arguments
/// * `global` - グローバルにインストールするかどうか。
/// * `shell` - インストールに使用するシェル（オプション）。
///
/// # Returns
/// `Ok(())` インストールが正常に完了した場合。
/// `Err(Error)` インストール中にエラーが発生した場合。
pub fn project_install(
    global: bool,
    shell: Option<ExecShell>,
) -> Result<(), Error> {
    let install_options = InstallOptions {
        install_shell: shell.unwrap_or_default(),
        install_mode: if global {
            ExecMode::Global
        } else {
            ExecMode::Local
        },
    };
    install::install(install_options).map_err(Error::from)
}

/// プロジェクトを削除します。
///
/// `remove`モジュールの`remove`関数を呼び出します。
///
/// # Arguments
/// * `remove_mode` - 削除モード。
/// * `shell` - 削除に使用するシェル（オプション）。
///
/// # Returns
/// `Ok(())` 削除が正常に完了した場合。
/// `Err(Error)` 削除中にエラーが発生した場合。
pub fn project_remove(
    remove_mode: ExecMode,
    shell: Option<ExecShell>,
) -> Result<(), Error> {
    let remove_options = RemoveOptions {
        remove_shell: shell.unwrap_or_default(),
        remove_mode,
    };
    remove::remove(remove_options).map_err(Error::from)
}

/// プロジェクトを完全に削除（パージ）します。
///
/// `purge`モジュールの`purge`関数を呼び出します。
///
/// # Arguments
/// * `purge_mode` - パージモード。
/// * `shell` - パージに使用するシェル（オプション）。
///
/// # Returns
/// `Ok(())` パージが正常に完了した場合。
/// `Err(Error)` パージ中にエラーが発生した場合。
pub fn project_purge(
    purge_mode: ExecMode,
    shell: Option<ExecShell>,
) -> Result<(), Error> {
    let purge_options = PurgeOptions {
        purge_shell: shell.unwrap_or_default(),
        purge_mode,
    };
    purge::purge(purge_options).map_err(Error::from)
}

/// プロジェクトのメタデータを表示します。
///
/// `metadata`モジュールの`show_metadata`関数を呼び出します。
///
/// # Returns
/// `Ok(())` メタデータが正常に表示された場合。
/// `Err(Error)` メタデータの表示中にエラーが発生した場合。
pub fn project_metadata() -> Result<(), Error> {
    metadata::show_metadata().map_err(Error::from)
}

/// 新しいプロジェクトを作成します。
///
/// `create`モジュールの`create`関数を呼び出します。
///
/// # Arguments
/// * `project_name` - プロジェクトの名前。
/// * `template` - 使用するテンプレート（オプション）。
/// * `author_name` - 著者名（オプション）。
/// * `author_email` - 著者メール（オプション）。
///
/// # Returns
/// `Ok(())` プロジェクトが正常に作成された場合。
/// `Err(Error)` プロジェクト作成中にエラーが発生した場合。
pub fn project_create(
    project_name: String,
    template: Option<ProjectTemplateType>,
    author_name: Option<String>,
    author_email: Option<String>,
) -> Result<(), Error> {
    let params = ProjectParams {
        project_name,
        project_template: template.unwrap_or_default(),
        author: AuthorAboutData {
            name: author_name.unwrap_or_else(username),
            email: author_email.unwrap_or_else(generate_email_address),
        },
    };
    println!("{}", params);

    fs::create_dir(&params.project_name)
        .map_err(|err| -> Error { err.into() })?;

    env::set_current_dir(&params.project_name)
        .map_err(|err| -> Error { err.into() })?;

    create::create(&params)
        .map_err(|_| {
            std::io::Error::other(format!(
                "failed to create project: {}",
                &params.project_name
            ))
        })
        .map_err(|err| -> Error { err.into() })?;

    Ok(())
}
