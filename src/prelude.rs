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
        use super::Error;
        use super::depend;
        use crate::modules::{pkg::*, project::ExecMode};
        use std::collections::HashMap;
        use std::path::PathBuf;

        // トレイトをスコープにインポート
        use super::depend::DependencyGraphOperations;

        /// パッケージのパスと解析済みのパッケージデータを保持する構造体です。
        #[derive(Clone)]
        pub struct PackageInfo {
            /// パッケージファイルのパス。
            pub path: PathBuf,
            /// 解析されたパッケージデータ。
            pub data: PackageData, // pkg::PackageData を格納
        }

        /// 指定したパスのパッケージをすべてインストールします (依存関係を考慮)。
        ///
        /// # 引数
        /// * `target` - インストールするパッケージファイルのパスのリスト。
        /// * `mode` - インストールモード（ローカルまたはグローバル）。
        ///
        /// # 返り値
        /// `Ok(())` - パッケージが正常にインストールされた場合。
        /// `Err(Error)` - エラーが発生した場合。
        pub fn install_packages(
            target: Vec<&PathBuf>,
            mode: ExecMode,
        ) -> Result<(), Error> {
            let mut package_infos: Vec<PackageInfo> =
                Vec::with_capacity(target.len());
            let mut package_info_map: HashMap<String, PackageInfo> =
                HashMap::new();

            for path in &target {
                if !path.is_file() {
                    return Err(Error::from(std::io::ErrorKind::NotFound));
                }

                let package_data = path.metadata()?;
                let pkg_info = PackageInfo {
                    path: path.to_path_buf(),
                    data: package_data,
                };
                package_info_map.insert(
                    pkg_info.data.about.package.name.clone(),
                    pkg_info.clone(),
                );
                package_infos.push(pkg_info);
            }

            let installed_packages = match mode {
                ExecMode::Global => list::get_global(),
                ExecMode::Local => list::get_local(),
            }?;

            let base_graph =
                depend::DependencyGraph::from_installed_packages(
                    &installed_packages,
                );

            let installing_package_data: Vec<PackageData> =
                package_infos.iter().map(|pi| pi.data.clone()).collect();

            let sorted_package_data = base_graph
                .topological_sort_packages_for_install(
                    &installing_package_data,
                )?;

            let sorted_package_infos: Vec<PackageInfo> =
                sorted_package_data
                    .iter()
                    .filter_map(|pkg_data| {
                        package_info_map
                            .remove(&pkg_data.about.package.name)
                    })
                    .collect();

            let temp_graph =
                base_graph.with_additional_packages(&sorted_package_data);

            temp_graph
                .is_packages_installable(sorted_package_data.clone())?;

            let file_paths: Vec<PathBuf> = sorted_package_infos.iter().map(|info| info.path.clone()).collect();
            install::install(&file_paths, mode)?;

            Ok(())
        }

        /// 指定した名称のパッケージをすべて削除します (依存関係を考慮)。
        ///
        /// # 引数
        /// * `target_names` - 削除するパッケージ名のリスト。
        /// * `mode` - 削除モード（ローカルまたはグローバル）。
        ///
        /// # 返り値
        /// `Ok(())` - パッケージが正常に削除された場合。
        /// `Err(Error)` - エラーが発生した場合。
        pub fn remove_packages(
            target_names: &Vec<String>,
            mode: ExecMode,
        ) -> Result<(), Error> {
            remove::remove(target_names, mode)
        }

        /// 指定したパッケージを設定ごと完全に削除（パージ）します (依存関係を考慮)。
        ///
        /// # 引数
        /// * `target_names` - パージするパッケージ名のリスト。
        /// * `mode` - パージモード（ローカルまたはグローバル）。
        ///
        /// # 返り値
        /// `Ok(())` - パッケージが正常にパージされた場合。
        /// `Err(Error)` - エラーが発生した場合。
        pub fn purge_packages(
            target_names: &Vec<String>,
            mode: ExecMode,
        ) -> Result<(), Error> {
            purge::purge(target_names, mode)
        }

        /// パスからパッケージのメタデータを取得するためのトレイトです。
        pub trait PackageMetadata {
            /// パスからパッケージのメタデータを取得します。
            ///
            /// # 引数
            /// なし (self)
            ///
            /// # 返り値
            /// `Ok(PackageData)` - メタデータが正常に取得された場合。
            /// `Err(Error)` - エラーが発生した場合。
            fn metadata(&self) -> Result<PackageData, Error>;
        }

        /// `PathBuf`に対する`PackageMetadata`トレイトの実装です。
        impl PackageMetadata for PathBuf {
            fn metadata(&self) -> Result<PackageData, Error> {
                metadata::get(self)
            }
        }

        /// パッケージのインストール処理をまとめるモジュールです。
        pub mod install {
            use super::{Error, ExecMode};
            use std::path::PathBuf;

            /// 指定したパッケージをインストールします。
            ///
            /// # 引数
            /// * `file_path` - インストールするパッケージファイルのパス。
            /// * `mode` - インストールモード（ローカルまたはグローバル）。
            ///
            /// # 返り値
            /// `Ok(())` - パッケージが正常にインストールされた場合。
            /// `Err(Error)` - エラーが発生した場合。
            pub fn install(
                file_paths: &Vec<PathBuf>,
                mode: ExecMode,
            ) -> Result<(), Error> {
                crate::modules::pkg::install::install(file_paths, mode)
            }
        }

        /// パッケージのアンインストール処理をまとめるモジュールです。
        pub mod uninstall {
            use super::Error;
            use crate::modules::project::ExecMode; // ExecMode を使用するために追加

            /// 指定したパッケージを削除します (ファイルのみ)。
            ///
            /// # 引数
            /// * `package_name` - 削除するパッケージの名前。
            /// * `mode` - 削除モード（ローカルまたはグローバル）。
            ///
            /// # 返り値
            /// `Ok(())` - パッケージが正常に削除された場合。
            /// `Err(Error)` - エラーが発生した場合。
            pub fn remove(
                package_name: String,
                mode: ExecMode,
            ) -> Result<(), Error> {
                println!(
                    "Removing package: {} in mode {:?}",
                    package_name, mode
                );
                // ここに実際のファイル削除ロジックを実装
                Ok(())
            }

            /// 指定したパッケージを設定ごと完全に削除（パージ）します (ファイルと設定)。
            ///
            /// # 引数
            /// * `package_name` - パージするパッケージの名前。
            /// * `mode` - パージモード（ローカルまたはグローバル）。
            ///
            /// # 返り値
            /// `Ok(())` - パッケージが正常にパージされた場合。
            /// `Err(Error)` - エラーが発生した場合。
            pub fn purge(
                package_name: String,
                mode: ExecMode,
            ) -> Result<(), Error> {
                println!(
                    "Purging package: {} in mode {:?}",
                    package_name, mode
                );
                // ここに実際のファイルと設定の削除ロジックを実装
                Ok(())
            }
        }
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
