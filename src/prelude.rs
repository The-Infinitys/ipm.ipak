//! prelude!
//! ipakの機能を利用しやすくまとめています。

pub mod ipak {
    use crate::utils::error::Error;
    /// バージョン構造体を利用できるようにしています。
    pub use crate::utils::version::{Version, VersionRange};
    use std::str::FromStr;
    /// ipak自身のバージョンを、Version構造体で返します。
    /// - 引数
    /// なし
    /// - 返り値
    /// ipakのバージョンを示す構造体
    pub fn version() -> Version {
        Version::from_str(
            option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
        ).expect("Failed to parse CARGO_PKG_VERSION. This is not a normal behavior.")
    }
    /// エラー系のコードをすぐに使用できるように
    pub use crate::utils::error;
    /// アーカイブ系統をまとめている
    pub mod archive {
        use super::Error;
        pub use crate::utils::archive::{self, ArchiveType};
        use std::{env, path::PathBuf};
        pub fn get_archive_type(
            path: &PathBuf,
        ) -> Result<ArchiveType, Error> {
            let target_path = env::current_dir()?.join(path);
            archive::get_archive_type(&target_path)
                .map_err(|e| Error::from(e))
        }
        pub fn create_archive(
            from: &PathBuf,
            to: &PathBuf,
            archive_type: ArchiveType,
        ) -> Result<(), Error> {
            archive::create_archive(from, to, archive_type)
                .map_err(|e| Error::from(e))
        }

        pub fn extract_archive(
            from: &PathBuf,
            to: &PathBuf,
        ) -> Result<(), Error> {
            archive::extract_archive(from, to).map_err(|e| Error::from(e))
        }
    }
    /// パッケージ系統の処理
    pub mod packages {
        use super::Error;
        use super::depend;
        use crate::modules::{pkg::*, project::ExecMode};
        use std::path::PathBuf;
        /// 指定したパスのパッケージをすべてインストールする (依存関係を考慮)
        pub fn install_packages(
            target: Vec<&PathBuf>,
            mode: ExecMode,
        ) -> Result<(), Error> {
            for path in &target {
                // ファイルの存在チェック
                if !path.is_file() {
                    return Err(Error::from(std::io::ErrorKind::NotFound));
                }
            }
            // メタデータをすべて取得
            let mut metadata_list = Vec::with_capacity(target.len());
            for path in &target {
                metadata_list.push(path.metadata()?)
            }
            let installed_packages = match mode {
                ExecMode::Global => list::get_global(),
                ExecMode::Local => list::get_local(),
            }?;
            let _graph = depend::DependencyGraph::from_installed_packages(
                &installed_packages,
            );

            // install(file_path, install_mode)
            Ok(())
        }
        /// 指定した名称のパッケージをすべて削除する (依存関係を考慮)
        pub fn remove_packages(_target: Vec<&str>) -> Result<(), Error> {
            Ok(())
        }
        /// 指定したパッケージを設定ごと削除する (依存関係を考慮)
        pub fn purge_packages(_target: Vec<&str>) -> Result<(), Error> {
            Ok(())
        }
        pub trait PackageMetadata {
            fn metadata(&self) -> Result<PackageData, Error>;
        }
        impl PackageMetadata for PathBuf {
            fn metadata(&self) -> Result<PackageData, Error> {
                metadata::get(self)
            }
        }
    }
    /// 引数系の処理
    pub mod args {
        use super::Error;
        pub use crate::utils::args::*;
        /// 指定したコマンドを実行し消費する
        pub trait CommandExecution {
            fn exec(self) -> Result<(), Error>;
        }
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
        impl CommandExecution for ProjectCommands {
            fn exec(self) -> Result<(), Error> {
                crate::modules::project::project(self)
            }
        }
        impl CommandExecution for SystemCommands {
            fn exec(self) -> Result<(), Error> {
                crate::modules::system::system(self)
            }
        }
        impl CommandExecution for PkgCommands {
            fn exec(self) -> Result<(), Error> {
                crate::modules::pkg::pkg(self)
            }
        }
        impl CommandExecution for UtilsCommands {
            fn exec(self) -> Result<(), Error> {
                crate::modules::utils::utils(self)
            }
        }
    }
    /// プロジェクト関連のモジュール
    pub mod project {
        // use super::Error;
    }
    /// 依存関係の解決モジュール

    pub mod depend {
        pub use crate::modules::pkg::depend::error::{
            InstallError, RemoveError,
        };
        pub use crate::modules::pkg::depend::graph::DependencyGraph;
        pub use crate::modules::pkg::depend::utils::get_missing_depend_cmds;
        pub use crate::modules::pkg::list::{
            InstalledPackageData, PackageListData,
        };
        pub use crate::modules::pkg::{
            PackageData, PackageRange, PackageVersion,
        };
        pub use crate::utils::version::{Version, VersionRange};
    }
}
