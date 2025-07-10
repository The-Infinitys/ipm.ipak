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

    pub mod packages {
        use super::Error;
        // use crate::modules::pkg;
        use std::path::PathBuf;
        /// 指定したパスのパッケージをすべてインストールする (依存関係を考慮)
        pub fn install_packages(
            _target: Vec<&PathBuf>,
        ) -> Result<(), Error> {
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
    }
    pub mod args {
        use super::Error;
        pub use crate::utils::args::*;

        pub trait CommandExecution {
            fn exec(self) -> Result<(), Error>;
        }
        impl CommandExecution for Commands {
            fn exec(self) -> Result<(), Error> {
                use crate::modules::{pkg, project, system, utils};
                match self {
                    Self::Project(args) => project::project(args),
                    Self::System(args) => system::system(args),
                    Self::Pkg(args) => pkg::pkg(args),
                    Self::Utils(args) => utils::utils(args),
                }
            }
        }
    }
}
