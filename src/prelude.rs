//! prelude!
//! ipakの機能を利用しやすくまとめています。

/// ipakクレートの主要な機能を提供します。
/// このモジュールをインポートすることで、ipakの様々なサブモジュールや関数に簡単にアクセスできます。
pub mod ipak {
    use crate::utils::error::Error;
    /// バージョン構造体を利用できるようにしています。
    pub use crate::utils::version::{Version, VersionRange};
    use std::str::FromStr;

    /// ipak自身のバージョンを、Version構造体で返します。
    ///
    /// # 引数
    /// なし
    ///
    /// # 返り値
    /// ipakのバージョンを示す構造体
    pub fn version() -> Version {
        Version::from_str(
            option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
        ).expect("Failed to parse CARGO_PKG_VERSION. This is not a normal behavior.")
    }

    /// エラー系のコードをすぐに使用できるようにします。
    pub use crate::utils::error;

    /// アーカイブ系統の処理をまとめています。
    pub mod archive {
        use super::Error;
        /// アーカイブ関連のユーティリティとアーカイブタイプを公開します。
        pub use crate::utils::archive::{self, ArchiveType};
        use std::{env, path::PathBuf};

        /// 指定したパスのアーカイブタイプを判定します。
        ///
        /// # 引数
        /// * `path` - 判定するアーカイブファイルのパス。
        ///
        /// # 返り値
        /// `Ok(ArchiveType)` - アーカイブタイプが正常に判定された場合。
        /// `Err(Error)` - エラーが発生した場合。
        pub fn get_archive_type(
            path: &PathBuf,
        ) -> Result<ArchiveType, Error> {
            let target_path = env::current_dir()?.join(path);
            archive::get_archive_type(&target_path).map_err(Error::from)
        }

        /// 指定したパスからアーカイブを作成します。
        ///
        /// # 引数
        /// * `from` - アーカイブ元となるパス。
        /// * `to` - 作成するアーカイブファイルの出力パス。
        /// * `archive_type` - 作成するアーカイブのタイプ。
        ///
        /// # 返り値
        /// `Ok(())` - アーカイブが正常に作成された場合。
        /// `Err(Error)` - エラーが発生した場合。
        pub fn create_archive(
            from: &PathBuf,
            to: &PathBuf,
            archive_type: ArchiveType,
        ) -> Result<(), Error> {
            archive::create_archive(from, to, archive_type)
                .map_err(Error::from)
        }

        /// 指定したアーカイブを解凍します。
        ///
        /// # 引数
        /// * `from` - 解凍するアーカイブファイルのパス。
        /// * `to` - 解凍先のディレクトリパス。
        ///
        /// # 返り値
        /// `Ok(())` - アーカイブが正常に解凍された場合。
        /// `Err(Error)` - エラーが発生した場合。
        pub fn extract_archive(
            from: &PathBuf,
            to: &PathBuf,
        ) -> Result<(), Error> {
            archive::extract_archive(from, to).map_err(Error::from)
        }
    }

    /// パッケージ系統の処理をまとめています。
    pub mod packages {
        use crate::modules::pkg::*;
        pub use install::install;
        pub use purge::purge;
        pub use remove::remove;
        pub use configure::configure;
    }

    /// 引数系の処理をまとめています。
    pub mod args {
        use super::Error;
        /// 引数関連のユーティリティを公開します。
        pub use crate::utils::args::*;

        /// 指定したコマンドを実行し消費するためのトレイトです。
        pub trait CommandExecution {
            /// コマンドを実行します。
            ///
            /// # 引数
            /// なし (self)
            ///
            /// # 返り値
            /// `Ok(())` - コマンドが正常に実行された場合。
            /// `Err(Error)` - エラーが発生した場合。
            fn exec(self) -> Result<(), Error>;
        }

        /// `Commands`列挙型に対する`CommandExecution`トレイトの実装です。
        impl CommandExecution for Commands {
            fn exec(self) -> Result<(), Error> {
                match self {
                    Self::Project(project_cmd) => project_cmd.exec(),
                    Self::System(system_cmd) => system_cmd.exec(),
                    Self::Pkg(pkg_cmd) => pkg_cmd.exec(),
                    Self::Utils(utils_cmd) => utils_cmd.exec(),
                }
            }
        }

        /// `ProjectCommands`列挙型に対する`CommandExecution`トレイトの実装です。
        impl CommandExecution for ProjectCommands {
            fn exec(self) -> Result<(), Error> {
                crate::modules::project::project(self)
            }
        }

        /// `SystemCommands`列挙型に対する`CommandExecution`トレイトの実装です。
        impl CommandExecution for SystemCommands {
            fn exec(self) -> Result<(), Error> {
                crate::modules::system::system(self)
            }
        }

        /// `PkgCommands`列挙型に対する`CommandExecution`トレイトの実装です。
        impl CommandExecution for PkgCommands {
            fn exec(self) -> Result<(), Error> {
                crate::modules::pkg::pkg(self)
            }
        }

        /// `UtilsCommands`列挙型に対する`CommandExecution`トレイトの実装です。
        impl CommandExecution for UtilsCommands {
            fn exec(self) -> Result<(), Error> {
                crate::modules::utils::utils(self)
            }
        }
    }

    /// プロジェクト関連のモジュールをまとめています。
    pub mod project {
        // src/modules/project.rs から主要なアイテムを再エクスポート
        /// 実行モード（ローカルまたはグローバル）を定義する列挙型です。
        pub use crate::modules::project::ExecMode;
        /// 実行に使用するシェルを定義する列挙型です。
        pub use crate::modules::project::ExecShell;
        /// プロジェクトテンプレートのタイプを定義する列挙型です。
        pub use crate::modules::project::ProjectTemplateType;
        /// プロジェクト関連のコマンドを処理するメインエントリポイントです。
        pub use crate::modules::project::project;
        /// プロジェクトをビルドします。
        pub use crate::modules::project::project_build;
        /// 新しいプロジェクトを作成します。
        pub use crate::modules::project::project_create;
        /// プロジェクトを初期化します。
        pub use crate::modules::project::project_init;
        /// プロジェクトをインストールします。
        pub use crate::modules::project::project_install;
        /// プロジェクトのメタデータを表示します。
        pub use crate::modules::project::project_metadata;
        /// プロジェクトをパッケージ化します。
        pub use crate::modules::project::project_package;
        /// プロジェクトを完全に削除（パージ）します。
        pub use crate::modules::project::project_purge;
        /// プロジェクトを削除します。
        pub use crate::modules::project::project_remove;
        /// プロジェクトを実行します。
        pub use crate::modules::project::project_run;

        // 関連するオプション構造体もエクスポート
        /// ビルドモード（リリースまたはデバッグ）を定義する列挙型です。
        pub use crate::modules::project::build::BuildMode;
        /// ビルドオプションを定義する構造体です。
        pub use crate::modules::project::build::BuildOptions;
        /// プロジェクト作成パラメータを定義する構造体です。
        pub use crate::modules::project::create::ProjectParams;
        /// インストールオプションを定義する構造体です。
        pub use crate::modules::project::install::InstallOptions;
        /// パッケージ化ターゲットを定義する列挙型です。
        pub use crate::modules::project::package::PackageTarget;
        /// パージオプションを定義する構造体です。
        pub use crate::modules::project::purge::PurgeOptions;
        /// 削除オプションを定義する構造体です。
        pub use crate::modules::project::remove::RemoveOptions;
    }

    /// 依存関係の解決モジュールをまとめています。
    pub mod depend {
        /// インストール関連のエラーを公開します。
        pub use crate::modules::pkg::depend::error::{
            InstallError, RemoveError,
        };
        /// 依存関係グラフの構造と操作を公開します。
        pub use crate::modules::pkg::depend::graph::{
            DependencyGraph, DependencyGraphOperations,
        };
        /// 不足している依存コマンドを取得するユーティリティを公開します。
        pub use crate::modules::pkg::depend::utils::get_missing_depend_cmds;

        /// インストール済みパッケージデータとパッケージリストデータを公開します。
        pub use crate::modules::pkg::list::{
            InstalledPackageData, PackageListData,
        };
        /// パッケージデータ、パッケージ範囲、パッケージバージョンを公開します。
        pub use crate::modules::pkg::{
            PackageData, PackageRange, PackageVersion,
        };
        /// バージョンとバージョン範囲を公開します。
        pub use crate::utils::version::{Version, VersionRange};
    }
}
