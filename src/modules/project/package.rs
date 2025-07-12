//! このモジュールは、プロジェクトを様々な形式でパッケージ化する機能を提供します。
//! プロジェクトのファイル構造を分析し、`.gitignore`のような設定を尊重して、
//! 指定されたターゲット（ソースビルド、通常、最小）に応じたアーカイブを作成します。

use super::metadata;
use crate::utils::archive::{ArchiveType, create_archive};
use crate::utils::color::colorize::*;
use ignore::gitignore::GitignoreBuilder;
use serde_yaml;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// パッケージ化のオプションを定義する構造体です。
#[derive(Debug, Default)]
pub struct PackageOptions {
    /// パッケージ化のターゲット（例: SourceBuild, Normal, Min）。
    pub target: PackageTarget,
}

/// パッケージ化のターゲットタイプを定義する列挙型です。
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum PackageTarget {
    /// ソースビルド用のパッケージ。
    SourceBuild,

    #[default]
    /// 通常のパッケージ。
    Normal,

    /// 最小限のパッケージ。
    Min,
}

impl Display for PackageTarget {
    /// `PackageTarget`を整形して表示します。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageTarget::SourceBuild => write!(f, "source-build"),
            PackageTarget::Normal => write!(f, "normal"),
            PackageTarget::Min => write!(f, "minimal"),
        }
    }
}

impl FromStr for PackageTarget {
    type Err = String;

    /// 文字列から`PackageTarget`をパースします。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "src" | "source" | "source-build" => Ok(Self::SourceBuild),
            "normal" | "default" => Ok(Self::Normal),
            "min" | "minimal" => Ok(Self::Min),
            _ => Err(format!("Invalid Package Target: {}", s)),
        }
    }
}

impl Display for PackageOptions {
    /// `PackageOptions`を整形して表示します。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", "Package Options".cyan().bold())?;
        writeln!(f, "  {}: {}", "target".green().bold(), self.target)?;
        Ok(())
    }
}

/// プロジェクトの無視設定を定義する構造体です。
#[derive(serde::Deserialize)]
struct ProjectIgnore {
    #[serde(rename = "source-build")]
    source_build: Vec<String>,
    normal: Vec<String>,
    min: Vec<String>,
}

/// 指定されたディレクトリの内容をコピーし、`.gitignore`パターンを尊重します。
///
/// # Arguments
/// * `source_base` - コピー元のベースディレクトリ。
/// * `dest_base` - コピー先のベースディレクトリ。
/// * `gitignore` - 使用する`.gitignore`パターン。
/// * `skip_prefix` - コピー時にスキップするパスのプレフィックス。
///
/// # Returns
/// `Ok(())` 成功した場合。
/// `Err(String)` コピー中にエラーが発生した場合。
fn walk_and_copy(
    source_base: &Path,
    dest_base: &Path,
    gitignore: &ignore::gitignore::Gitignore,
    skip_prefix: &Path,
) -> Result<(), String> {
    fn inner(
        dir: &Path,
        source_base: &Path,
        dest_base: &Path,
        gitignore: &ignore::gitignore::Gitignore,
        skip_prefix: &Path,
    ) -> Result<(), String> {
        let dir_rel = dir.strip_prefix(source_base).map_err(|_| {
            format!("Failed to get relative path for directory {:?}", dir)
        })?;
        if dir_rel.starts_with(skip_prefix) {
            return Ok(());
        }

        for entry in fs::read_dir(dir).map_err(|e| {
            format!("Failed to read directory {:?}: {}", dir, e)
        })? {
            let entry = entry
                .map_err(|e| format!("Failed to get entry: {}", e))?;
            let path = entry.path();
            let path_rel =
                path.strip_prefix(source_base).map_err(|_| {
                    format!("Failed to get relative path for {:?}", path)
                })?;

            if path.is_dir() {
                inner(
                    &path,
                    source_base,
                    dest_base,
                    gitignore,
                    skip_prefix,
                )?;
            } else if gitignore.matched(path_rel, true).is_ignore() {
                log::debug!("Ignored: {}", path_rel.display());
            } else {
                let dest = dest_base.join(path_rel);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        format!(
                            "Failed to create directories for {:?}: {}",
                            parent, e
                        )
                    })?;
                }
                fs::copy(&path, &dest).map_err(|e| {
                    format!(
                        "Failed to copy {:?} to {:?}: {}",
                        path, dest, e
                    )
                })?;
                log::debug!(
                    "Copied {} to {}",
                    path.display(),
                    dest.display()
                );
            }
        }

        Ok(())
    }

    inner(source_base, source_base, dest_base, gitignore, skip_prefix)
}

/// プロジェクトをパッケージ化します。
///
/// 指定されたパッケージオプションに基づいて、プロジェクトのファイルを収集し、
/// `.gitignore`設定を尊重してアーカイブを作成します。
///
/// # Arguments
/// * `opts` - パッケージ化オプションを含む`PackageOptions`構造体。
///
/// # Returns
/// `Ok(())` パッケージ化が正常に完了した場合。
/// `Err(String)` パッケージ化中にエラーが発生した場合。
pub fn package(opts: PackageOptions) -> Result<(), String> {
    log::debug!("Starting packaging process with options: {}", &opts);

    let target_dir = metadata::get_dir().map_err(|e| {
        format!(
            "Error: Couldn't find Ipak Directory. Make sure you are in an ipak project. Details: {:?}", 
            e
        )
    })?;
    log::debug!("Project directory: {}", target_dir.display());

    let project_metadata = metadata::metadata().map_err(|e| {
        format!("Error: Failed to read project metadata: {:?}", e)
    })?;
    log::debug!(
        "Project metadata loaded for: {} version {}",
        project_metadata.about.package.name,
        project_metadata.about.package.version
    );

    let ignore_file = target_dir.join("ipak").join("project-ignore.yaml");
    let ignore_config: ProjectIgnore = if ignore_file.exists() {
        let file = fs::File::open(&ignore_file).map_err(|e| {
            format!("Failed to open '{}': {}", ignore_file.display(), e)
        })?;
        serde_yaml::from_reader(file).map_err(|e| {
            format!("Failed to parse '{}': {}", ignore_file.display(), e)
        })?
    } else {
        log::debug!(
            "Warning: '{}' not found, using empty ignore lists",
            ignore_file.display()
        );
        ProjectIgnore { source_build: vec![], normal: vec![], min: vec![] }
    };

    let ignore_list: Vec<String> = match opts.target {
        PackageTarget::SourceBuild => ignore_config.source_build,
        PackageTarget::Normal => {
            let mut list = ignore_config.source_build;
            list.extend(ignore_config.normal);
            list
        }
        PackageTarget::Min => {
            let mut list = ignore_config.source_build;
            list.extend(ignore_config.normal);
            list.extend(ignore_config.min);
            list
        }
    };

    log::debug!(
        "Ignore list for target {}: [\n{}\n]",
        opts.target,
        ignore_list.join("\n")
    );
    log::debug!("Target Directory: {}", target_dir.display());

    let mut builder = GitignoreBuilder::new(&target_dir);
    for pattern in &ignore_list {
        if let Err(e) = builder.add_line(None, pattern.as_str()) {
            log::error!("Error: {}", e)
        };
        log::debug!("Adding ignore pattern: {}", pattern);
    }
    let gitignore = builder
        .build()
        .map_err(|e| format!("Failed to build gitignore: {}", e))?;
    log::debug!("Gitignore built: {}", gitignore.len());

    let source_base = &target_dir;
    let package_name = &project_metadata.about.package.name;
    let version = &project_metadata.about.package.version;
    let dest_base: PathBuf = source_base
        .join("ipak")
        .join("package")
        .join(format!("{}-{}/", package_name, version));
    let skip_prefix: PathBuf = PathBuf::from("ipak").join("package");

    walk_and_copy(source_base, &dest_base, &gitignore, &skip_prefix)?;

    let archive_path: PathBuf = source_base
        .join("ipak")
        .join("package")
        .join(format!("{}-{}.ipak", package_name, version));

    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!("Failed to create directories for {:?}: {}", parent, e)
        })?;
    }

    log::debug!("Creating zip archive at {}", archive_path.display());
    create_archive(&dest_base, &archive_path, ArchiveType::Zip)
        .map_err(|e| format!("Failed to create archive: {}", e))?;

    fs::remove_dir_all(&dest_base).map_err(|e| {
        format!("Failed to remove directory {:?}: {}", dest_base, e)
    })?;
    log::debug!("Removed temporary directory {}", dest_base.display());

    if !archive_path.exists() {
        return Err(format!(
            "Archive file {} was not created",
            archive_path.display()
        ));
    }

    log::debug!("Created archive at {}", archive_path.display());

    Ok(())
}
