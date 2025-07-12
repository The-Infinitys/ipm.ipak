//! このモジュールは、コマンドライン引数の解析を定義します。
//! `clap`クレートを使用して、アプリケーションの様々なコマンドとサブコマンドを構造化します。

use crate::modules::project::ProjectTemplateType;
use crate::modules::project::package::PackageTarget;
use crate::{modules::project::ExecShell, utils::archive::ArchiveType};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "A command-line tool for easy project creation, building, and installation. / プロジェクトの作成、ビルド、インストールを簡単に行えるツール。", long_about = None)]
#[command(name = "ipak")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage projects. / プロジェクトを管理します。
    #[command(subcommand)]
    Project(ProjectCommands),
    /// Configure system settings. / システム設定を構成します。
    #[command(subcommand)]
    System(SystemCommands),
    /// Utility commands. / ユーティリティコマンド。
    #[command(subcommand)]
    Utils(UtilsCommands),
    /// Manage packages. / パッケージを管理します。
    #[command(subcommand)]
    Pkg(PkgCommands),
}
#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    /// Create a new project. / 新しいプロジェクトを作成します。
    Create {
        /// Name of the project. / プロジェクトの名前。
        #[arg(long = "project-name")]
        project_name: String,
        /// Template to use (e.g., default, rust). / 使用するテンプレート (例: default, rust)。
        #[arg(long)]
        template: Option<ProjectTemplateType>,
        /// Author name for the project. / プロジェクトの著者名。
        #[arg(long)]
        author_name: Option<String>,
        /// Author email for the project. / プロジェクトの著者メール。
        #[arg(long)]
        author_email: Option<String>,
    },
    /// Build the project. / プロジェクトをビルドします。
    Build {
        /// Build in release mode. / リリースモードでビルドします。
        #[arg(long)]
        release: bool,
        /// Shell to use (e.g., bash, zsh). / 使用するシェル (例: bash, zsh)。
        #[arg(long)]
        shell: Option<ExecShell>,
    },
    /// Install the project. / プロジェクトをインストールします。
    Install {
        /// Install globally. / グローバルにインストールします。
        #[arg(long)]
        global: bool,
        /// Shell to use (e.g., bash, zsh). / 使用するシェル (例: bash, zsh)。
        #[arg(long)]
        shell: Option<ExecShell>,
    },
    /// Remove the project. / プロジェクトを削除します。
    Remove {
        /// Remove locally. / ローカルで削除します。
        #[arg(long)]
        local: bool,
        /// Remove globally. / グローバルで削除します。
        #[arg(long)]
        global: bool,
        /// Shell to use (e.g., bash, zsh). / 使用するシェル (例: bash, zsh)。
        #[arg(long)]
        shell: Option<ExecShell>,
    },
    /// Completely remove the project and associated data. / プロジェクトと関連データを完全に削除します。
    Purge {
        /// Purge locally. / ローカルで完全に削除します。
        #[arg(long)]
        local: bool,
        /// Purge globally. / グローバルで完全に削除します。
        #[arg(long)]
        global: bool,
        /// Shell to use (e.g., bash, zsh). / 使用するシェル (例: bash, zsh)。
        #[arg(long)]
        shell: Option<ExecShell>,
    },
    /// Package the project. / プロジェクトをパッケージ化します。
    Package {
        /// Target for packaging (e.g., "tar", "zip"). / パッケージ化のターゲット (例: "tar", "zip")。
        #[arg(long)]
        target: Option<PackageTarget>,
    },
    /// Display project metadata. / プロジェクトのメタデータを表示します。
    Metadata,
    /// Initialize a project. / プロジェクトを初期化します。
    Init,
    /// Run an arbitrary command within the project. / プロジェクト内で任意のコマンドを実行します。
    Run {
        /// Shell to run. / 実行するシェル。
        shell: Option<ExecShell>,
        /// Command to run. / 実行するコマンド。
        command: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SystemCommands {
    /// Configure local or global settings. / ローカルまたはグローバル設定を構成します。
    Configure {
        /// Configure local settings. / ローカル設定を構成します。
        #[arg(long)]
        local: bool,
        /// Configure global settings. / グローバル設定を構成します。
        #[arg(long)]
        global: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum UtilsCommands {
    /// Archive utilities. / アーカイブユーティリティ。
    #[command(subcommand)]
    Archive(ArchiveCommands),
}

#[derive(Subcommand, Debug)]
pub enum ArchiveCommands {
    /// Create an archive. / アーカイブを作成します。
    Create {
        /// Path to the target to archive. / アーカイブするターゲットへのパス。
        #[arg(long)]
        from: PathBuf,
        /// Path to create the archive. / アーカイブを作成するパス。
        #[arg(long)]
        to: PathBuf,
        /// Type of archive (e.g., tar, zstd). / アーカイブの種類 (例: tar, zstd)。
        #[arg(long, value_name = "TYPE")]
        archive_type: ArchiveType,
    },
    /// Extract an archive. / アーカイブを展開します。
    Extract {
        /// Path to the archive to extract. / 展開するアーカイブへのパス。
        #[arg(long)]
        from: PathBuf,
        /// Path to extract the archive to. / アーカイブを展開するパス。
        #[arg(long)]
        to: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum PkgCommands {
    /// List installed packages. / インストール済みパッケージを一覧表示します。
    List {
        /// List local packages. / ローカルパッケージを一覧表示します。
        #[arg(long)]
        local: bool,
        /// List global packages. / グローバルパッケージを一覧表示します。
        #[arg(long)]
        global: bool,
    },
    /// Install a package. / パッケージをインストールします。
    Install {
        /// Path to the package file. / パッケージファイルへのパス。
        #[arg()]
        file_paths: Vec<PathBuf>,
        /// Install locally. / ローカルにインストールします。
        #[arg(long)]
        local: bool,
        /// Install globally. / グローバルにインストールします。
        #[arg(long)]
        global: bool,
    },
    /// Remove a package (binaries only, config files are kept). / パッケージを削除します。バイナリのみが削除され、設定ファイルは残ったままになります。
    Remove {
        /// Name of the package to remove. / 削除するパッケージの名前。
        #[arg()]
        package_names: Vec<String>,
        /// Remove locally. / ローカルで削除します。
        #[arg(long)]
        local: bool,
        /// Remove globally. / グローバルで削除します。
        #[arg(long)]
        global: bool,
    },
    /// Purge a package (completely removed, including config files). / パッケージを削除します。設定ファイルも含めて完全に削除されます。
    Purge {
        /// Name of the package to purge. / 完全に削除するパッケージの名前。
        #[arg()]
        package_names: Vec<String>,
        /// Purge locally. / ローカルで完全に削除します。
        #[arg(long)]
        local: bool,
        /// Purge globally. / グローバルで完全に削除します。
        #[arg(long)]
        global: bool,
    },
    /// Display package metadata. / パッケージのメタデータを表示します。
    MetaData {
        /// Path to the package to get metadata from. / メタデータを取得するパッケージへのパス。
        #[arg()]
        package_path: PathBuf,
    },
    /// Configure a package. / パッケージを設定します。
    Configure {
        /// Name of the package to configure. / 設定するパッケージの名前。
        #[arg()]
        package_names: Vec<String>,
        /// Configure locally. / ローカルで設定します。
        #[arg(long)]
        local: bool,
        /// Configure globally. / グローバルで設定します。
        #[arg(long)]
        global: bool,
    },
}
