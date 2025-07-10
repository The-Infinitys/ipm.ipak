//! このモジュールは、プロジェクトのビルドに関連する機能を提供します。
//! プロジェクトのメタデータと指定されたビルドモードに基づいて、プロジェクトをビルドします。

use super::ExecShell;
use super::metadata::{self, metadata};
use crate::dprintln;
use crate::utils::version::Version;
use crate::utils::color::colorize::*;
use std::fmt::{self, Display};
use std::process::Command;

/// プロジェクトビルドのオプションを定義する構造体です。
#[derive(Default)]
pub struct BuildOptions {
    /// ビルドモード（リリースまたはデバッグ）。
    pub build_mode: BuildMode,
    /// ビルドに使用するシェル。
    pub build_shell: ExecShell,
}

impl Display for BuildOptions {
    /// `BuildOptions`を整形して表示します。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = [
            format!("{}{}", "Build Options".cyan().bold(), ":"),
            format!(
                "  {}{} {}",
                "build-mode".green().bold(),
                ":",
                self.build_mode
            ),
            format!(
                "  {}{} {}",
                "build-shell".green().bold(),
                ":",
                self.build_shell
            ),
        ];
        for line in lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

/// ビルドモードを定義する列挙型です。
#[derive(Default)]
pub enum BuildMode {
    /// リリースモードでのビルド。
    Release,
    #[default]
    /// デバッグモードでのビルド。
    Debug,
}

impl Display for BuildMode {
    /// `BuildMode`を整形して表示します。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildMode::Release => {
                write!(f, "release")
            }
            BuildMode::Debug => {
                write!(f, "debug")
            }
        }
    }
}

/// プロジェクトをビルドします。
///
/// 指定されたビルドオプションに基づいて、プロジェクトをビルドします。
/// ビルドは`ipak/scripts/build.sh`スクリプトを通じて実行されます。
///
/// # Arguments
/// * `opts` - ビルドオプションを含む`BuildOptions`構造体。
///
/// # Returns
/// `Ok(())` ビルドが正常に完了した場合。
/// `Err(String)` ビルド中にエラーが発生した場合。
pub fn build(opts: BuildOptions) -> Result<(), String> {
    dprintln!("{}", &opts);
    let target_dir = metadata::get_dir().map_err(|e| format!("Error: {}", e))?;
    let project_metadata = metadata().map_err(|e| format!("Error: {}", e))?;

    fn setup_execshell(
        cmd: &mut Command,
        target_dir: &std::path::Path,
        project_name: &str,
        project_version: &Version,
        build_mode: &BuildMode,
    ) {
        let build_mode = build_mode.to_string();
        cmd.current_dir(target_dir)
            .env("IPAK_PROJECT_NAME", project_name)
            .env("IPAK_PROJECT_VERSION", project_version.to_string())
            .env("IPAK_BUILD_MODE", build_mode)
            .arg("ipak/scripts/build.sh");
    }

    let mut build_process = opts.build_shell.generate();
    setup_execshell(
        &mut build_process,
        &target_dir,
        &project_metadata.about.package.name,
        &project_metadata.about.package.version,
        &opts.build_mode,
    );

    let status = build_process
        .status()
        .map_err(|e| format!("Failed to execute build process: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Build process failed with status: {}", status))
    }
}
