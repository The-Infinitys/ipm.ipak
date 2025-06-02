use super::metadata;
use crate::dprintln;
use crate::utils::shell::is_cmd_available;
use colored::Colorize;
use ignore::gitignore::GitignoreBuilder;
use serde_yaml;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

/// Defines the options for the packaging process.
#[derive(Debug, Default)]
pub struct PackageOptions {
    /// The target type for the package (e.g., source build, normal, minimal).
    pub target: PackageTarget,
}

/// Represents the different packaging targets.
#[derive(Debug, Clone, Copy, Default)]
pub enum PackageTarget {
    /// Builds from source.
    SourceBuild,
    /// Standard package.
    #[default]
    Normal,
    /// Minimal package.
    Min,
}

impl Display for PackageTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageTarget::SourceBuild => {
                write!(f, "source-build")
            }
            PackageTarget::Normal => {
                write!(f, "normal")
            }
            PackageTarget::Min => {
                write!(f, "minimal")
            }
        }
    }
}

impl FromStr for PackageTarget {
    type Err = String;

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", "Package Options".cyan().bold())?;
        writeln!(f, "  {}: {}", "target".green().bold(), self.target)?;
        Ok(())
    }
}

/// Structure to deserialize project-ignore.yaml
#[derive(serde::Deserialize)]
struct ProjectIgnore {
    #[serde(rename = "source-build")]
    source_build: Vec<String>,
    normal: Vec<String>,
    min: Vec<String>,
}

/// Function to recursively walk and copy files from source to destination,
/// while respecting the ignore list and skipping specified prefixes.
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
        // Compute the relative path of the current directory
        let dir_rel = dir.strip_prefix(source_base).map_err(|_| {
            format!("Failed to get relative path for directory {:?}", dir)
        })?;
        // If the current directory's relative path starts with the skip prefix,
        // skip this directory and its contents
        if dir_rel.starts_with(skip_prefix) {
            return Ok(());
        }

        // Iterate over all entries in the current directory
        for entry in fs::read_dir(dir).map_err(|e| {
            format!("Failed to read directory {:?}: {}", dir, e)
        })? {
            let entry = entry
                .map_err(|e| format!("Failed to get entry: {}", e))?;
            let path = entry.path();

            // Compute the relative path of the current file or directory
            let path_rel =
                path.strip_prefix(source_base).map_err(|_| {
                    format!("Failed to get relative path for {:?}", path)
                })?;

            if path.is_dir() {
                // Recurse into subdirectories
                inner(
                    &path,
                    source_base,
                    dest_base,
                    gitignore,
                    skip_prefix,
                )?;
            } else {
                // Check if the file is ignored
                // Convert path to a relative path from source_base
                // let path =
                //     path.strip_prefix(source_base).map_err(|_| {
                //         format!(
                //             "Failed to get relative path for {:?}",
                //             path
                //         )
                //     })?;

                // Check if the file is ignored using the relative path

                if gitignore.matched(path_rel, true).is_ignore() {
                    dprintln!("Ignored: {}", path_rel.display());
                } else {
                    // Compute the destination path
                    let dest = dest_base.join(path_rel);

                    // Ensure the parent directories of the destination exist
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            format!("Failed to create directories for {:?}: {}", parent, e)
                        })?;
                    }

                    // Copy the file to the destination
                    fs::copy(&path, &dest).map_err(|e| {
                        format!(
                            "Failed to copy {:?} to {:?}: {}",
                            path, dest, e
                        )
                    })?;
                    dprintln!(
                        "Copied {} to {}",
                        path.display(),
                        dest.display()
                    );
                }
            }
        }

        Ok(())
    }

    // Start the recursive walk from the source base
    inner(source_base, source_base, dest_base, gitignore, skip_prefix)
}

/// Initiates the packaging process based on the provided options.
pub fn package(opts: PackageOptions) -> Result<(), String> {
    dprintln!("Starting packaging process with options: {}", &opts);

    // Get project directory
    let target_dir = metadata::get_dir().map_err(|e| {
        format!(
            "Error: Couldn't find Ipak Directory. Make sure you are in an ipak project. Details: {:?}", 
            e
        )
    })?;
    dprintln!("Project directory: {}", target_dir.display());

    // Load project metadata
    let project_metadata = metadata::metadata().map_err(|e| {
        format!("Error: Failed to read project metadata: {:?}", e)
    })?;
    dprintln!(
        "Project metadata loaded for: {} version {}",
        project_metadata.about.package.name,
        project_metadata.about.package.version
    );

    // Load project-ignore.yaml
    let ignore_file = target_dir.join("ipak").join("project-ignore.yaml");
    let ignore_config: ProjectIgnore = if ignore_file.exists() {
        let file = std::fs::File::open(&ignore_file).map_err(|e| {
            format!("Failed to open '{}': {}", ignore_file.display(), e)
        })?;
        serde_yaml::from_reader(file).map_err(|e| {
            format!("Failed to parse '{}': {}", ignore_file.display(), e)
        })?
    } else {
        dprintln!(
            "Warning: '{}' not found, using empty ignore lists",
            ignore_file.display()
        );
        ProjectIgnore { source_build: vec![], normal: vec![], min: vec![] }
    };

    // Select ignore list based on PackageTarget
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

    dprintln!(
        "Ignore list for target {}: [\n{}\n]",
        opts.target,
        ignore_list.join("\n")
    );

    // Build Gitignore from the ignore list
    let mut builder = GitignoreBuilder::new(&target_dir);
    for pattern in &ignore_list {
        builder.add(target_dir.join(pattern.as_str()));
    }
    let gitignore = builder
        .build()
        .map_err(|e| format!("Failed to build gitignore: {}", e))?;

    // Define source base, destination base, and skip prefix
    let source_base = &target_dir;
    let package_name = &project_metadata.about.package.name;
    let version = &project_metadata.about.package.version;
    let dest_base: PathBuf = source_base
        .join("ipak")
        .join("package")
        .join(format!("{}-{}", package_name, version));
    let skip_prefix: PathBuf = PathBuf::from("ipak").join("package");

    // Call walk_and_copy to perform the file copying
    walk_and_copy(source_base, &dest_base, &gitignore, &skip_prefix)?;

    // Compress the destination directory into a tar.gz archive
    let archive_path: PathBuf = source_base
        .join("ipak")
        .join("package")
        .join(format!("{}-{}.ipak", package_name, version));

    // Ensure the parent directory of the archive exists
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!("Failed to create directories for {:?}: {}", parent, e)
        })?;
    }

    // Check if tar command is available
    if !is_cmd_available("tar") {
        return Err(
            "tarコマンドが利用できません。tarがインストールされていることを確認してください。"
                .to_string(),
        );
    }

    // Run tar command to create a tar.gz archive
    let tar_output = Command::new("tar")
        .arg("-czf")
        .arg(&archive_path)
        .arg("-C")
        .arg(
            dest_base.parent().ok_or(
                "dest_baseの親ディレクトリを取得できませんでした",
            )?,
        )
        .arg(
            dest_base
                .file_name()
                .ok_or("ディレクトリ名を取得できませんでした")?,
        )
        .output()
        .map_err(|e| format!("tarコマンドの実行に失敗しました: {}", e))?;

    if !tar_output.status.success() {
        return Err(format!(
            "tarコマンドが失敗しました: {}",
            String::from_utf8_lossy(&tar_output.stderr)
        ));
    }

    dprintln!("アーカイブを {} に作成しました", archive_path.display());

    // Clean up the destination directory after compression
    fs::remove_dir_all(&dest_base).map_err(|e| {
        format!("ディレクトリ {:?} の削除に失敗しました: {}", dest_base, e)
    })?;
    dprintln!("一時ディレクトリ {} を削除しました", dest_base.display());

    // Verify the archive exists
    if !archive_path.exists() {
        return Err(format!(
            "アーカイブファイル {} が作成されませんでした",
            archive_path.display()
        ));
    }

    Ok(())
}
